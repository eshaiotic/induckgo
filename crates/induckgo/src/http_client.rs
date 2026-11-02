//! Plain HTTP downloads made by the browser itself (tracker lists, web app manifests and icons).
//!
//! It uses libsoup, the HTTP stack WebKitGTK already ships, so InDuckGo adds no second TLS implementation. The
//! session has no cookie jar: these requests carry no identity. Every body is read with a size limit, since a
//! hostile server could otherwise stream forever.

use std::error::Error;
use std::fmt;

use gio::prelude::InputStreamExt;
use soup::prelude::SessionExt;

pub enum Fetched {
    /// The server confirmed the copy matching the given `ETag` is still current.
    NotModified,
    Body {
        bytes: Vec<u8>,
        etag: Option<String>,
    },
}

#[derive(Debug)]
pub enum FetchError {
    Network(glib::Error),
    Status(soup::Status, String),
    TooLarge(String),
    InvalidAddress(String),
}

impl fmt::Display for FetchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(error) => write!(formatter, "{error}"),
            Self::Status(status, url) => write!(formatter, "{url} answered {status:?}"),
            Self::TooLarge(url) => write!(formatter, "{url} is larger than allowed"),
            Self::InvalidAddress(url) => write!(formatter, "{url} is not a valid address"),
        }
    }
}

impl Error for FetchError {}

pub struct HttpClient {
    session: soup::Session,
}

impl HttpClient {
    const TIMEOUT_SECONDS: u32 = 30;
    const CHUNK_BYTES: usize = 64 * 1024;

    pub fn new() -> Self {
        let session = soup::Session::new();
        session.set_timeout(Self::TIMEOUT_SECONDS);
        Self { session }
    }

    /// `GET url`, revalidating with `etag` when given; the body may be at most `max_bytes` long.
    pub async fn get(&self, url: &str, max_bytes: usize, etag: Option<&str>) -> Result<Fetched, FetchError> {
        let message = soup::Message::new("GET", url).map_err(|_| FetchError::InvalidAddress(url.to_owned()))?;
        if let (Some(etag), Some(headers)) = (etag, message.request_headers()) {
            headers.append("If-None-Match", etag);
        }
        let stream = self.session.send_future(&message, glib::Priority::DEFAULT).await.map_err(FetchError::Network)?;
        match message.status() {
            soup::Status::NotModified if etag.is_some() => Ok(Fetched::NotModified),
            soup::Status::Ok => {
                let bytes = Self::read_limited(&stream, url, max_bytes).await?;
                let etag = message.response_headers().and_then(|headers| headers.one("ETag")).map(String::from);
                Ok(Fetched::Body { bytes, etag })
            }
            status => Err(FetchError::Status(status, url.to_owned())),
        }
    }

    async fn read_limited(stream: &gio::InputStream, url: &str, max_bytes: usize) -> Result<Vec<u8>, FetchError> {
        let mut body = Vec::new();
        loop {
            let chunk = stream
                .read_bytes_future(Self::CHUNK_BYTES, glib::Priority::DEFAULT)
                .await
                .map_err(FetchError::Network)?;
            if chunk.is_empty() {
                return Ok(body);
            }
            if body.len() + chunk.len() > max_bytes {
                return Err(FetchError::TooLarge(url.to_owned()));
            }
            body.extend_from_slice(&chunk);
        }
    }
}
