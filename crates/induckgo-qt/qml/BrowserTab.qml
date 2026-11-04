import QtQuick
import QtWebEngine

WebEngineView {
    id: view
    property string startAddress
    property string smoothWheelSource
    readonly property string address: url.toString()

    url: startAddress
    settings.javascriptCanOpenWindows: false

    userScripts.collection: smoothWheelSource === "" ? [] : [ {
        name: "induckgo-smooth-wheel",
        sourceCode: smoothWheelSource,
        injectionPoint: WebEngineScript.DocumentCreation,
        worldId: WebEngineScript.ApplicationWorld,
        runsOnSubFrames: true
    } ]

    function load(destination) { url = destination }
}
