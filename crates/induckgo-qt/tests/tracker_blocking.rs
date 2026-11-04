//! The real application, started headless, blocks a tracker that DuckDuckGo's data lists and lets the rest load.
//!
//! The test serves a page from `first.test` that embeds one image from `tracker.test` (listed as a blocked
//! tracker in a fixture data set) and one from `other.test`. Chromium resolves every `.test` name to the local
//! server, which records which names were requested.

use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use serde_json::json;

// The browser's Qt bridge lives in this library; the test starts the binary, but links it too.
use induckgo_qt as _;

/// Host names of the requests the local server received, in order.
type RequestedHosts = Arc<Mutex<Vec<String>>>;

struct BrowsingScenario {
    port: u16,
    requested: RequestedHosts,
    home: tempdir::ScratchDirectory,
}

mod tempdir {
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;

    /// A directory under the system's temporary directory that is removed when dropped.
    pub struct ScratchDirectory {
        path: PathBuf,
    }

    impl ScratchDirectory {
        pub fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
            fs::create_dir_all(&path).expect("scratch directory");
            Self { path }
        }

        pub fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for ScratchDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

impl BrowsingScenario {
    /// Given: `tracker.test` is a tracker the data set blocks by default, and a local server answers for every
    /// `.test` name.
    fn given_a_tracker_listed_in_the_downloaded_data() -> Self {
        let home = tempdir::ScratchDirectory::new("induckgo-qt-test");
        Self::write_protection_cache(&home.path().join("cache/induckgo/protections"));
        let listener = TcpListener::bind("127.0.0.1:0").expect("listen");
        let port = listener.local_addr().expect("address").port();
        let requested: RequestedHosts = Arc::default();
        let recorder = Arc::clone(&requested);
        thread::spawn(move || Self::serve(&listener, &recorder));
        Self { port, requested, home }
    }

    fn write_protection_cache(directory: &Path) {
        fs::create_dir_all(directory).expect("cache directory");
        let tracker_data = json!({
            "trackers": {"tracker.test": {"domain": "tracker.test", "owner": {"name": "Tracker Inc"}, "default": "block", "rules": []}},
            "entities": {"Tracker Inc": {"domains": ["tracker.test"]}},
            "cnames": {}
        });
        // The local server speaks plain HTTP, so the fixture exempts the test names from the HTTPS upgrade.
        let exceptions: Vec<_> =
            ["first.test", "other.test", "tracker.test"].iter().map(|domain| json!({"domain": domain})).collect();
        let configuration = json!({"features": {"https": {"state": "enabled", "exceptions": exceptions}}});
        fs::write(directory.join("tracker-data.json"), tracker_data.to_string()).expect("tracker data");
        fs::write(directory.join("configuration.json"), configuration.to_string()).expect("configuration");
    }

    fn serve(listener: &TcpListener, requested: &RequestedHosts) {
        // One thread per connection: Chromium opens speculative connections that never send a request.
        for stream in listener.incoming().flatten() {
            let requested = Arc::clone(requested);
            thread::spawn(move || Self::answer(stream, &requested));
        }
    }

    fn answer(mut stream: TcpStream, requested: &RequestedHosts) {
        let (path, host) = Self::read_request(&stream);
        // Chromium opens speculative connections that never send a request.
        if path.is_empty() {
            return;
        }
        if path != "/favicon.ico" {
            requested.lock().expect("lock").push(host);
        }
        let port = stream.local_addr().map_or(0, |address| address.port());
        let page = if path == "/" {
            format!(
                "<html><body>hi<img src=\"http://tracker.test:{port}/pixel.gif\"><img src=\"http://other.test:{port}/pixel.gif\"></body></html>"
            )
        } else {
            "GIF89a".to_owned()
        };
        let head = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            page.len()
        );
        let _ = stream.write_all(head.as_bytes());
        let _ = stream.write_all(page.as_bytes());
    }

    /// Returns the request path and the `Host` header's name (without port).
    fn read_request(stream: &TcpStream) -> (String, String) {
        let mut reader = BufReader::new(stream);
        let mut host = String::new();
        let mut path = String::new();
        let mut line = String::new();
        while reader.read_line(&mut line).is_ok_and(|bytes| bytes > 0) {
            let header = line.trim_end().to_ascii_lowercase();
            if header.is_empty() {
                break;
            }
            if path.is_empty() {
                header.split_whitespace().nth(1).unwrap_or_default().clone_into(&mut path);
            } else if let Some(value) = header.strip_prefix("host:") {
                value.trim().split(':').next().unwrap_or_default().clone_into(&mut host);
            }
            line.clear();
        }
        (path, host)
    }

    fn when_the_page_is_opened_in_the_browser(&self) -> BrowserProcess {
        let home = self.home.path();
        let child = Command::new(env!("CARGO_BIN_EXE_induckgo-qt"))
            .arg(format!("http://first.test:{}/", self.port))
            .env("QT_QPA_PLATFORM", "offscreen")
            .env("XDG_DATA_HOME", home.join("data"))
            .env("XDG_CACHE_HOME", home.join("cache"))
            .env("QTWEBENGINE_CHROMIUM_FLAGS", "--host-resolver-rules=\"MAP *.test 127.0.0.1\"")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("start the browser");
        BrowserProcess { child }
    }

    fn requested_hosts_after_the_other_image_loaded(&self) -> Vec<String> {
        let deadline = Instant::now() + Duration::from_secs(60);
        while Instant::now() < deadline && !self.requested.lock().expect("lock").iter().any(|host| host == "other.test")
        {
            thread::sleep(Duration::from_millis(100));
        }
        thread::sleep(Duration::from_secs(2));
        self.requested.lock().expect("lock").clone()
    }
}

/// The running browser, stopped when the scenario ends.
struct BrowserProcess {
    child: Child,
}

impl Drop for BrowserProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn should_load_the_page_and_the_ordinary_image_but_not_the_tracker_image() {
    let scenario = BrowsingScenario::given_a_tracker_listed_in_the_downloaded_data();
    let _browser = scenario.when_the_page_is_opened_in_the_browser();
    let hosts = scenario.requested_hosts_after_the_other_image_loaded();
    assert!(hosts.contains(&"first.test".to_owned()), "the page itself should load: {hosts:?}");
    assert!(hosts.contains(&"other.test".to_owned()), "an unlisted third party should load: {hosts:?}");
    assert!(!hosts.contains(&"tracker.test".to_owned()), "the tracker should be blocked: {hosts:?}");
}
