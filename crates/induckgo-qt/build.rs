use cxx_qt_build::CxxQtBuilder;
use cxx_qt_build::QmlModule;

fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("io.induckgo.browser").qml_files(["qml/Main.qml", "qml/BrowserTab.qml"]),
    )
    .files(["src/browser_bridge.rs"])
    .qt_module("Quick")
    .qt_module("Gui")
    .build();
}
