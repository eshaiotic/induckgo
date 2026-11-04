import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// The whole download history: search by name, remove one entry, or clear the history.
Dialog {
    id: manager
    required property var hub
    required property var bridge

    title: bridge.message("Downloads")
    modal: true
    padding: 16
    width: Math.min(640, parent ? parent.width - 48 : 640)
    height: Math.min(560, parent ? parent.height - 48 : 560)
    anchors.centerIn: parent
    standardButtons: Dialog.Close
    onAboutToShow: { hub.managerOpen = true; hub.reload() }
    onClosed: hub.managerOpen = false

    ColumnLayout {
        anchors.fill: parent
        spacing: 8

        RowLayout {
            Layout.fillWidth: true
            TextField {
                id: search
                Layout.fillWidth: true
                placeholderText: manager.bridge.message("Search Downloads")
                selectByMouse: true
            }
            Button {
                text: manager.bridge.message("Clear History")
                enabled: manager.hub.everything.count > 0
                onClicked: manager.hub.clearHistory()
            }
        }
        ListView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: manager.hub.everything
            delegate: DownloadRow {
                bridge: manager.bridge
                removable: true
                onRemoveRequested: function(id) { manager.hub.remove(id) }
                visible: search.text === "" || name.toLowerCase().includes(search.text.toLowerCase())
                height: visible ? implicitHeight : 0
                onShowInFolder: function(path) { manager.hub.showInFolder(path) }
            }
            Label {
                anchors.centerIn: parent
                visible: parent.count === 0
                text: manager.bridge.message("No downloads yet")
                color: palette.placeholderText
            }
            ScrollBar.vertical: ScrollBar { }
        }
    }
}
