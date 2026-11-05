import QtQuick
import QtWebEngine
import QtWebChannel
import io.induckgo.browser

// One page: a Chromium view with the scripts every page gets.
WebEngineView {
    id: view
    property string startAddress
    property string smoothWheelSource
    property string extensionPromptSource
    // Qt's web-channel client followed by Guard's page script; empty when the browser has none to give.
    property string guardSource
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
        if (guardSource !== "") scripts.push(userScript("induckgo-guard-page-hiding", guardSource))
        return scripts
    }

    // The page script reaches Rust through a channel of its own isolated world, which the page cannot see.
    webChannel: WebChannel { registeredObjects: [guardBridge] }
    webChannelWorld: WebEngineScript.ApplicationWorld
    GuardBridge { id: guardBridge; WebChannel.id: "guard" }

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
