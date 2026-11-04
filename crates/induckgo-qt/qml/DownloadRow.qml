import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// One download in the list: its name, how far it has got or what became of it, and a button for its folder.
ItemDelegate {
    id: row
    required property var bridge
    required property int id
    required property string name
    required property string path
    required property string state
    required property real received
    required property real total
    required property bool missing

    property bool removable: false

    signal showInFolder(string path)
    signal removeRequested(int id)

    width: ListView.view ? ListView.view.width : implicitWidth
    implicitHeight: 56
    hoverEnabled: true
    onClicked: if (state === "finished" && !missing) row.showInFolder(path)

    contentItem: RowLayout {
        spacing: 10
        Image {
            Layout.preferredWidth: 22
            Layout.preferredHeight: 22
            sourceSize: Qt.size(22, 22)
            source: "qrc:/icons/download.svg"
            opacity: 0.7
        }
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 2
            Text {
                Layout.fillWidth: true
                text: row.name
                textFormat: Text.PlainText
                elide: Text.ElideMiddle
                color: row.palette.windowText
            }
            Text {
                Layout.fillWidth: true
                visible: row.state !== "progress"
                text: row.state === "failed" ? row.bridge.message("Download failed")
                    : row.missing ? row.bridge.message("File Moved or Deleted") : ""
                textFormat: Text.PlainText
                color: row.palette.placeholderText
                font.pixelSize: 12
            }
            ProgressBar {
                Layout.fillWidth: true
                visible: row.state === "progress"
                indeterminate: row.total <= 0
                value: row.total > 0 ? row.received / row.total : 0
            }
            Text {
                visible: row.state === "progress"
                text: row.total > 0
                    ? row.bridge.message("Downloading… {percent}%").replace("{percent}", Math.floor(100 * row.received / row.total))
                    : row.bridge.message("Downloading…")
                color: row.palette.placeholderText
                font.pixelSize: 12
            }
        }
        IconButton {
            iconName: "folder"
            tip: row.bridge.message("Show in Folder")
            visible: row.state === "finished"
            onClicked: row.showInFolder(row.path)
        }
        IconButton {
            iconName: "trash"
            tip: row.bridge.message("Remove from List")
            visible: row.removable
            onClicked: row.removeRequested(row.id)
        }
    }
}
