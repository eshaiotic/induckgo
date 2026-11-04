import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtWebEngine
import io.induckgo.browser

ApplicationWindow {
    id: window
    width: 1280
    height: 800
    visible: true
    title: currentTab ? currentTab.title + " — InDuckGo" : "InDuckGo"

    BrowserBridge { id: bridge }

    readonly property var currentTab: tabs.count > 0 ? tabStack.children[tabs.currentIndex] : null

    function openTab(address) {
        const tab = tabComponent.createObject(tabStack, { startAddress: address || bridge.home_page() })
        tabs.addTab(tab)
        tabs.currentIndex = tabs.count - 1
        return tab
    }

    function closeTab(index) {
        if (tabs.count <= 1) { Qt.quit(); return }
        const tab = tabStack.children[index]
        tabs.removeTab(index)
        tab.destroy()
    }

    header: ColumnLayout {
        spacing: 0
        ToolBar {
            Layout.fillWidth: true
            RowLayout {
                anchors.fill: parent
                ToolButton {
                    icon.name: "go-previous"
                    text: bridge.message("Back")
                    display: AbstractButton.IconOnly
                    enabled: window.currentTab && window.currentTab.canGoBack
                    onClicked: window.currentTab.goBack()
                    ToolTip.visible: hovered
                    ToolTip.text: text
                }
                ToolButton {
                    icon.name: "go-next"
                    text: bridge.message("Forward")
                    display: AbstractButton.IconOnly
                    enabled: window.currentTab && window.currentTab.canGoForward
                    onClicked: window.currentTab.goForward()
                    ToolTip.visible: hovered
                    ToolTip.text: text
                }
                ToolButton {
                    icon.name: "view-refresh"
                    text: bridge.message("Reload")
                    display: AbstractButton.IconOnly
                    onClicked: window.currentTab.reload()
                    ToolTip.visible: hovered
                    ToolTip.text: text
                }
                TextField {
                    id: addressBar
                    Layout.fillWidth: true
                    selectByMouse: true
                    placeholderText: bridge.message("Search or enter address")
                    text: window.currentTab ? window.currentTab.address : ""
                    onAccepted: {
                        const destination = bridge.destination(text)
                        if (destination !== "") window.currentTab.load(destination)
                        window.currentTab.forceActiveFocus()
                    }
                }
                ToolButton {
                    icon.name: "tab-new"
                    text: bridge.message("New Tab")
                    display: AbstractButton.IconOnly
                    onClicked: window.openTab("")
                    ToolTip.visible: hovered
                    ToolTip.text: text
                }
            }
        }
        TabBar {
            id: tabs
            Layout.fillWidth: true
            function addTab(tab) {
                const button = tabButton.createObject(tabs, { tab: tab })
                tabs.addItem(button)
            }
        }
        Label {
            Layout.fillWidth: true
            visible: text !== ""
            padding: 8
            wrapMode: Text.Wrap
            background: Rectangle { color: "#fff3cd" }
            text: bridge.startup_notice()
        }
    }

    Component {
        id: tabButton
        TabButton {
            property var tab
            width: Math.min(240, implicitWidth)
            text: tab ? (tab.title || tab.address) : ""
            onClicked: tabs.currentIndex = TabBar.index
        }
    }

    Component {
        id: tabComponent
        BrowserTab { smoothWheelSource: bridge.smooth_wheel_script() }
    }

    StackLayout {
        id: tabStack
        anchors.fill: parent
        currentIndex: tabs.currentIndex
    }

    Shortcut { sequences: ["Ctrl+T"]; onActivated: window.openTab("") }
    Shortcut { sequences: ["Ctrl+W"]; onActivated: window.closeTab(tabs.currentIndex) }
    Shortcut { sequences: ["Ctrl+L"]; onActivated: { addressBar.forceActiveFocus(); addressBar.selectAll() } }
    Shortcut { sequences: ["Ctrl+R", "F5"]; onActivated: window.currentTab.reload() }
    Shortcut { sequences: ["Alt+Left"]; onActivated: window.currentTab.goBack() }
    Shortcut { sequences: ["Alt+Right"]; onActivated: window.currentTab.goForward() }

    Component.onCompleted: window.openTab(Qt.application.arguments.length > 1 ? bridge.destination(Qt.application.arguments[1]) : "")
}
