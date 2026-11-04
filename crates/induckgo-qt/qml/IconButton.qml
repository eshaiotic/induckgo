import QtQuick
import QtQuick.Controls

// A flat toolbar button that shows one of the application's icons, tinted with the theme's text colour.
ToolButton {
    id: control
    property string iconName
    property string tip

    flat: true
    display: AbstractButton.IconOnly
    icon.source: iconName === "" ? "" : "qrc:/icons/" + iconName + ".svg"
    icon.width: 20
    icon.height: 20
    icon.color: enabled ? palette.buttonText : palette.mid
    implicitWidth: 36
    implicitHeight: 36
    Accessible.name: tip
    ToolTip.visible: hovered && tip !== ""
    ToolTip.delay: 500
    ToolTip.text: tip

    background: Rectangle {
        radius: 6
        color: control.down ? Qt.alpha(control.palette.buttonText, 0.18)
             : control.hovered ? Qt.alpha(control.palette.buttonText, 0.09) : "transparent"
    }
}
