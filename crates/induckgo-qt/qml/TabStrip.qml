import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// The row of open tabs: favicon, title, and a close button on each; shown only when there is more than one.
Rectangle {
    id: strip
    property var tabs: []
    property int currentIndex: 0
    property string closeTip

    signal selected(int index)
    signal closeRequested(int index)

    color: palette.window
    implicitHeight: 38

    Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: Qt.alpha(palette.windowText, 0.15) }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: 6
        anchors.rightMargin: 6
        anchors.topMargin: 4
        spacing: 2

        Repeater {
            model: strip.tabs.length
            delegate: Rectangle {
                id: tabItem
                required property int index
                readonly property var tab: strip.tabs[index]
                readonly property bool current: index === strip.currentIndex

                Layout.fillWidth: true
                Layout.maximumWidth: 240
                Layout.minimumWidth: 72
                Layout.fillHeight: true
                radius: 8
                color: current ? palette.base : hover.hovered ? Qt.alpha(palette.windowText, 0.07) : "transparent"
                border.color: current ? Qt.alpha(palette.windowText, 0.18) : "transparent"

                HoverHandler { id: hover }
                TapHandler {
                    acceptedButtons: Qt.LeftButton
                    onTapped: strip.selected(tabItem.index)
                }
                TapHandler {
                    acceptedButtons: Qt.MiddleButton
                    onTapped: strip.closeRequested(tabItem.index)
                }

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 10
                    anchors.rightMargin: 4
                    spacing: 8

                    Image {
                        Layout.preferredWidth: 16
                        Layout.preferredHeight: 16
                        sourceSize: Qt.size(16, 16)
                        source: tabItem.tab ? tabItem.tab.icon : ""
                        visible: status === Image.Ready
                    }
                    // The title comes from the page: show it as plain text, never as markup.
                    Text {
                        Layout.fillWidth: true
                        text: tabItem.tab ? (tabItem.tab.title || tabItem.tab.address) : ""
                        textFormat: Text.PlainText
                        elide: Text.ElideRight
                        color: palette.windowText
                        font.weight: tabItem.current ? Font.DemiBold : Font.Normal
                    }
                    IconButton {
                        iconName: "close"
                        tip: strip.closeTip
                        implicitWidth: 24
                        implicitHeight: 24
                        icon.width: 14
                        icon.height: 14
                        opacity: tabItem.current || hover.hovered ? 1 : 0
                        onClicked: strip.closeRequested(tabItem.index)
                    }
                }
            }
        }
        Item { Layout.fillWidth: true; Layout.minimumWidth: 0; visible: strip.tabs.length < 4 }
    }
}
