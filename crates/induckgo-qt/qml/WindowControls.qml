import QtQuick
import QtQuick.Controls

// Minimise, maximise or restore, and close: the window buttons of a window that draws its own title bar, as
// libadwaita draws them (round, 24 px, a little apart). They carry no tooltips, like GNOME's own.
Row {
    id: controls
    required property var window
    readonly property bool isMaximized: window.visibility === Window.Maximized

    spacing: 13

    component Control: AbstractButton {
        id: button
        property string iconName
        implicitWidth: 24
        implicitHeight: 24
        background: Rectangle {
            radius: 12
            color: button.down ? Qt.alpha(palette.windowText, 0.19)
                 : button.hovered ? Qt.alpha(palette.windowText, 0.14) : Qt.alpha(palette.windowText, 0.08)
        }
        contentItem: Item {
            TintedIcon { anchors.centerIn: parent; size: 16; name: button.iconName }
        }
    }

    Control { iconName: "minimize"; onClicked: controls.window.showMinimized() }
    Control {
        iconName: controls.isMaximized ? "restore" : "maximize"
        onClicked: controls.isMaximized ? controls.window.showNormal() : controls.window.showMaximized()
    }
    Control { iconName: "close"; onClicked: controls.window.close() }
}
