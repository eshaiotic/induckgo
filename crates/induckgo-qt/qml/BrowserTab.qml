import QtQuick
import QtWebEngine

// One page: a Chromium view with the scripts every page gets.
WebEngineView {
    id: view
    property string startAddress
    property string smoothWheelSource
    property string extensionPromptSource
    readonly property string address: url.toString()
    readonly property bool isLoading: loading

    /// A link wants a new tab (target="_blank", middle click).
    signal openInNewTab(string address)

    url: startAddress
    settings.javascriptCanOpenWindows: false
    // A page loaded from file:// must not read other local files, and a local page must not reach the network.
    settings.localContentCanAccessFileUrls: false
    settings.localContentCanAccessRemoteUrls: false

    userScripts.collection: {
        const scripts = []
        if (smoothWheelSource !== "") scripts.push(userScript("induckgo-smooth-wheel", smoothWheelSource))
        if (extensionPromptSource !== "") scripts.push(userScript("induckgo-duckduckgo-extension-prompt", extensionPromptSource))
        return scripts
    }

    function userScript(scriptName, source) {
        return {
            name: scriptName,
            sourceCode: source,
            injectionPoint: WebEngineScript.DocumentCreation,
            worldId: WebEngineScript.ApplicationWorld,
            runsOnSubFrames: true
        }
    }

    onNewWindowRequested: function(request) {
        if (request.userInitiated) view.openInNewTab(request.requestedUrl.toString())
    }

    function load(destination) { url = destination }
}
