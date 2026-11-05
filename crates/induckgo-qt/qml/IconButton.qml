import QtQuick
import QtQuick.Controls

// A flat header-bar button that shows one of the application's icons in the theme's text colour. `checked`
// (a toggle such as the shield) shows the button pressed in, as libadwaita draws a toggled flat button.
ToolButton {
    id: control
    property string iconName
    property string tip

    flat: true
    padding: 0
    display: AbstractButton.IconOnly
    icon.source: iconName === "" ? "" : "qrc:/icons/" + iconName + ".svg"
    icon.width: 16
    icon.height: 16
    icon.color: palette.buttonText
    opacity: enabled ? 1.0 : 0.35
    implicitWidth: 34
    implicitHeight: 34
    Accessible.name: tip
    ToolTip.visible: hovered && tip !== ""
    ToolTip.delay: 500
    ToolTip.text: tip

    background: Rectangle {
        radius: 9
        color: control.down ? Qt.alpha(palette.windowText, 0.19)
             : control.checked ? Qt.alpha(palette.windowText, 0.12)
             : control.hovered ? Qt.alpha(palette.windowText, 0.08) : "transparent"
    }
}
