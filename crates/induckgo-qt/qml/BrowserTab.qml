import QtQuick
import QtWebEngine

// One page: a Chromium view with the scripts every page gets.
WebEngineView {
    id: view
    property string startAddress
    property string smoothWheelSource
    readonly property string address: url.toString()
    readonly property bool isLoading: loading

    /// A link wants a new tab (target="_blank", middle click).
    signal openInNewTab(string address)

    url: startAddress
    settings.javascriptCanOpenWindows: false
    // A page loaded from file:// must not read other local files, and a local page must not reach the network.
    settings.localContentCanAccessFileUrls: false
    settings.localContentCanAccessRemoteUrls: false

    userScripts.collection: smoothWheelSource === "" ? [] : [ {
        name: "induckgo-smooth-wheel",
        sourceCode: smoothWheelSource,
        injectionPoint: WebEngineScript.DocumentCreation,
        worldId: WebEngineScript.ApplicationWorld,
        runsOnSubFrames: true
    } ]

    onNewWindowRequested: function(request) {
        if (request.userInitiated) view.openInNewTab(request.requestedUrl.toString())
    }

    function load(destination) { url = destination }
}
