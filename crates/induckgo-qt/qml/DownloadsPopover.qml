import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// The toolbar's downloads list: the five latest, then the manager, "Clear List" and the download folder.
Popup {
    id: popover
    required property var hub
    required property var bridge
    signal managerRequested()

    width: 380
    padding: 6
    modal: false
    focus: true
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    background: Rectangle {
        radius: 8
        color: palette.window
        border.color: Qt.alpha(palette.windowText, 0.25)
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 2

        Label {
            visible: popover.hub.latest.count === 0
            Layout.fillWidth: true
            Layout.margins: 16
            horizontalAlignment: Text.AlignHCenter
            text: popover.bridge.message("No downloads yet")
            color: palette.placeholderText
        }
        ListView {
            Layout.fillWidth: true
            Layout.preferredHeight: contentHeight
            visible: popover.hub.latest.count > 0
            interactive: false
            model: popover.hub.latest
            delegate: DownloadRow {
                bridge: popover.bridge
                onShowInFolder: function(path) { popover.hub.showInFolder(path); popover.close() }
            }
        }
        MenuSeparator { Layout.fillWidth: true }
        ItemDelegate {
            Layout.fillWidth: true
            text: popover.bridge.message("Show All Downloads")
            onClicked: { popover.close(); popover.managerRequested() }
        }
        ItemDelegate {
            Layout.fillWidth: true
            text: popover.bridge.message("Clear List")
            enabled: popover.hub.latest.count > 0
            onClicked: popover.hub.clearList()
        }
        ItemDelegate {
            Layout.fillWidth: true
            text: popover.bridge.message("Go to Download Folder")
            onClicked: { popover.hub.openFolder(); popover.close() }
        }
    }
}
