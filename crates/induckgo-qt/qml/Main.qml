import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtWebEngine
import QtQuick.Dialogs
import io.induckgo.browser

ApplicationWindow {
    id: window
    width: 1280
    height: 860
    visible: true
    title: currentTab && currentTab.title ? currentTab.title + " — InDuckGo" : "InDuckGo"

    BrowserBridge { id: bridge }

    property var tabList: []
    property int currentIndex: 0
    readonly property var currentTab: tabList.length > 0 ? tabList[currentIndex] : null
    readonly property string currentHost: currentTab ? bridge.host_of(currentTab.address) : ""
    readonly property int currentZoom: currentTab ? bridge.zoom_percent(currentTab.address) : 100

    // "_Burn" in the catalogue marks the button's keyboard letter; the dialog's own buttons need none.
    function plain(text) { return bridge.message(text).replace("_", "") }

    function openTab(address) {
        const tab = tabComponent.createObject(tabStack, { startAddress: address || bridge.home_page() })
        tab.openInNewTab.connect(function(target) { window.openTab(target) })
        tab.urlChanged.connect(function() { window.applyZoom(tab) })
        tabList = tabList.concat([tab])
        currentIndex = tabList.length - 1
        if (!address) addressBar.forceActiveFocus()
        return tab
    }

    function closeTab(index) {
        if (tabList.length <= 1) { Qt.quit(); return }
        const tab = tabList[index]
        tabList = tabList.filter(function(_, position) { return position !== index })
        currentIndex = Math.min(index < currentIndex ? currentIndex - 1 : currentIndex, tabList.length - 1)
        tab.destroy()
    }

    function applyZoom(tab) { tab.zoomFactor = bridge.zoom_percent(tab.address) / 100 }

    function changeZoom(direction) {
        if (!currentTab) return
        bridge.change_zoom(currentTab.address, direction)
        tabList.forEach(applyZoom)
        zoomVersion++
    }
    property int zoomVersion: 0

    function toast(text) { toastLabel.text = text; toastPopup.open(); toastTimer.restart() }

    function navigate(typed) {
        const destination = bridge.destination(typed)
        if (destination !== "" && currentTab) {
            currentTab.load(destination)
            currentTab.forceActiveFocus()
        }
    }

    function burn() {
        const error = bridge.burn_and_restart()
        if (error === "") Qt.quit(); else toast(error)
    }

    // Every command exists once, as an Action: the menu, the toolbar and the keyboard all run the same one.
    Action {
        id: newTab
        text: bridge.message("New Tab")
        icon.source: "qrc:/icons/add.svg"
        shortcut: "Ctrl+T"
        onTriggered: window.openTab("")
    }
    Action {
        id: closeTabAction
        shortcut: "Ctrl+W"
        onTriggered: window.closeTab(window.currentIndex)
    }
    Action {
        id: showDownloads
        text: bridge.message("Downloads")
        icon.source: "qrc:/icons/download.svg"
        shortcut: "Ctrl+J"
        onTriggered: downloadsPopover.opened ? downloadsPopover.close() : downloadsPopover.open()
    }
    Action {
        id: findInPage
        text: bridge.message("Find in Page…")
        icon.source: "qrc:/icons/search.svg"
        shortcut: "Ctrl+F"
        onTriggered: findBar.open()
    }
    Action {
        id: focusAddress
        shortcut: "Ctrl+L"
        onTriggered: { addressBar.forceActiveFocus(); addressBar.selectAll() }
    }
    Action { id: reloadAction; shortcut: "Ctrl+R"; onTriggered: window.currentTab.reload() }
    Action { id: reloadFunctionKey; shortcut: "F5"; onTriggered: window.currentTab.reload() }
    Action { id: goBack; shortcut: "Alt+Left"; onTriggered: window.currentTab.goBack() }
    Action { id: goForward; shortcut: "Alt+Right"; onTriggered: window.currentTab.goForward() }
    Action { shortcut: "Ctrl++"; onTriggered: window.changeZoom(1) }
    Action { shortcut: "Ctrl+="; onTriggered: window.changeZoom(1) }
    Action { shortcut: "Ctrl+-"; onTriggered: window.changeZoom(-1) }
    Action { shortcut: "Ctrl+0"; onTriggered: window.changeZoom(0) }
    Action {
        id: burnAction
        text: bridge.message("Burn Browsing Data")
        icon.source: "qrc:/icons/fire.svg"
        shortcut: "Ctrl+Shift+Delete"
        onTriggered: burnDialog.open()
    }
    Action {
        id: aboutAction
        text: bridge.message("About InDuckGo")
        icon.source: "qrc:/icons/info.svg"
        onTriggered: aboutDialog.open()
    }
    Action {
        id: preferencesAction
        text: bridge.message("Preferences…")
        icon.source: "qrc:/icons/settings.svg"
        onTriggered: preferencesDialog.open()
    }

    header: ColumnLayout {
        spacing: 0

        ToolBar {
            Layout.fillWidth: true
            Layout.preferredHeight: 52
            background: Rectangle {
                color: palette.window
                Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: Qt.alpha(palette.windowText, 0.12) }
            }

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 8
                anchors.rightMargin: 8
                spacing: 2

                IconButton {
                    iconName: "back"
                    tip: bridge.message("Back")
                    enabled: window.currentTab && window.currentTab.canGoBack
                    onClicked: window.currentTab.goBack()
                }
                IconButton {
                    iconName: "forward"
                    tip: bridge.message("Forward")
                    enabled: window.currentTab && window.currentTab.canGoForward
                    onClicked: window.currentTab.goForward()
                }
                IconButton {
                    iconName: window.currentTab && window.currentTab.isLoading ? "close" : "reload"
                    tip: window.currentTab && window.currentTab.isLoading ? bridge.message("Load the page again, or stop loading it") : bridge.message("Reload")
                    onClicked: window.currentTab.isLoading ? window.currentTab.stop() : window.currentTab.reload()
                }

                Item { Layout.preferredWidth: 6 }

                Rectangle {
                    id: addressField
                    Layout.fillWidth: true
                    Layout.maximumWidth: 900
                    Layout.alignment: Qt.AlignHCenter
                    Layout.preferredHeight: 36
                    radius: 18
                    color: palette.base
                    border.width: addressBar.activeFocus ? 2 : 1
                    border.color: addressBar.activeFocus ? palette.highlight : Qt.alpha(palette.windowText, 0.22)
                    clip: true

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 12
                        anchors.rightMargin: 6
                        spacing: 6

                        Image {
                            readonly property string scheme: window.currentTab ? window.currentTab.address.split(":")[0] : ""
                            Layout.preferredWidth: 16
                            Layout.preferredHeight: 16
                            sourceSize: Qt.size(16, 16)
                            visible: scheme === "https" || scheme === "http"
                            source: scheme === "https" ? "qrc:/icons/lock.svg" : "qrc:/icons/warning.svg"
                            opacity: 0.7
                        }
                        TextField {
                            id: addressBar
                            Layout.fillWidth: true
                            background: null
                            selectByMouse: true
                            placeholderText: bridge.message("Search {engine} or enter address").replace("{engine}", "DuckDuckGo")
                            text: window.currentTab ? window.currentTab.address : ""
                            onAccepted: window.navigate(text)
                            onActiveFocusChanged: if (activeFocus) selectAll()
                        }
                        Button {
                            id: zoomBadge
                            visible: window.currentZoom !== 100 && window.zoomVersion >= 0
                            flat: true
                            text: window.currentZoom + "%"
                            implicitHeight: 26
                            ToolTip.visible: hovered
                            ToolTip.text: bridge.message("Reset Zoom")
                            onClicked: window.changeZoom(0)
                        }
                    }
                    ProgressBar {
                        anchors.bottom: parent.bottom
                        anchors.left: parent.left
                        anchors.right: parent.right
                        height: 3
                        visible: window.currentTab && window.currentTab.isLoading
                        value: window.currentTab ? window.currentTab.loadProgress / 100 : 0
                        background: null
                    }
                }

                Item { Layout.preferredWidth: 6 }

                IconButton {
                    id: shieldButton
                    checkable: true
                    enabled: window.currentHost !== ""
                    checked: window.currentHost === "" || bridge.is_site_protected(window.currentHost)
                    iconName: checked ? "shield-on" : "shield-off"
                    tip: bridge.message("Tracker protection on this site")
                    onClicked: {
                        bridge.set_site_protected(window.currentHost, checked)
                        window.currentTab.reload()
                    }
                }
                IconButton {
                    iconName: "fire"
                    tip: bridge.message("Burn Browsing Data")
                    onClicked: burnAction.trigger()
                }
                IconButton {
                    iconName: "add"
                    tip: bridge.message("New Tab")
                    onClicked: newTab.trigger()
                }
                IconButton {
                    id: downloadsButton
                    iconName: "download"
                    tip: bridge.message("Downloads")
                    onClicked: showDownloads.trigger()

                    DownloadsPopover {
                        id: downloadsPopover
                        hub: downloads
                        bridge: bridge
                        y: parent.height + 4
                        x: parent.width - width
                        onManagerRequested: downloadsManager.open()
                    }
                }
                IconButton {
                    iconName: "menu"
                    tip: bridge.message("Main Menu")
                    onClicked: mainMenu.popup(x, y + height)

                    Menu {
                        id: mainMenu
                        width: 300
                        MenuItem { action: newTab }
                        MenuItem { action: showDownloads }
                        MenuItem { action: findInPage }
                        MenuSeparator { }
                        Item {
                            width: parent.width
                            height: 44
                            RowLayout {
                                anchors.centerIn: parent
                                spacing: 8
                                IconButton { iconName: "minus"; tip: bridge.message("Zoom Out"); onClicked: window.changeZoom(-1) }
                                Button {
                                    flat: true
                                    implicitWidth: 72
                                    text: window.currentZoom + "%"
                                    ToolTip.visible: hovered
                                    ToolTip.text: bridge.message("Reset Zoom")
                                    onClicked: window.changeZoom(0)
                                }
                                IconButton { iconName: "add"; tip: bridge.message("Zoom In"); onClicked: window.changeZoom(1) }
                            }
                        }
                        MenuSeparator { }
                        MenuItem { action: preferencesAction }
                        MenuItem { action: aboutAction }
                    }
                }
            }
        }

        TabStrip {
            Layout.fillWidth: true
            visible: window.tabList.length > 1
            Layout.preferredHeight: visible ? 38 : 0
            tabs: window.tabList
            currentIndex: window.currentIndex
            closeTip: ""
            onSelected: function(index) { window.currentIndex = index }
            onCloseRequested: function(index) { window.closeTab(index) }
        }

        Label {
            Layout.fillWidth: true
            visible: text !== ""
            padding: 8
            wrapMode: Text.Wrap
            textFormat: Text.PlainText
            background: Rectangle { color: "#fff3cd" }
            color: "#533f03"
            text: bridge.startup_notice()
        }
    }

    Component {
        id: tabComponent
        BrowserTab { smoothWheelSource: bridge.smooth_wheel_script() }
    }

    StackLayout {
        id: tabStack
        anchors.fill: parent
        currentIndex: window.currentIndex
    }

    footer: FindBar {
        id: findBar
        view: window.currentTab
        placeholder: bridge.message("Find in page")
        closeTip: ""
        onClosed: if (window.currentTab) window.currentTab.forceActiveFocus()
    }

    DownloadsHub {
        id: downloads
        bridge: bridge
        onFinished: function(name) { window.toast(bridge.message("Downloaded {name}").replace("{name}", name)) }
        onFailed: function(message) { window.toast(message) }
    }
    DownloadsManager { id: downloadsManager; hub: downloads; bridge: bridge }

    Dialog {
        id: burnDialog
        title: bridge.message("Burn Browsing Data?")
        modal: true
        padding: 20
        anchors.centerIn: parent
        width: Math.min(460, window.width - 48)
        footer: DialogButtonBox {
            Button { text: window.plain("_Cancel"); DialogButtonBox.buttonRole: DialogButtonBox.RejectRole }
            Button { text: window.plain("_Burn"); highlighted: true; DialogButtonBox.buttonRole: DialogButtonBox.AcceptRole }
        }
        onAccepted: window.burn()
        Label {
            width: parent.width
            wrapMode: Text.Wrap
            text: bridge.message("Closes all tabs and windows of this profile and deletes its cookies, site data, cache and history.")
        }
    }

    Dialog {
        id: aboutDialog
        title: bridge.message("About InDuckGo")
        modal: true
        padding: 20
        anchors.centerIn: parent
        standardButtons: Dialog.Close
        ColumnLayout {
            spacing: 6
            Label { text: "InDuckGo " + Qt.application.version; font.bold: true; font.pixelSize: 18 }
            Label { text: "GPL-3.0-or-later"; color: palette.placeholderText }
        }
    }

    Dialog {
        id: preferencesDialog
        title: bridge.message("Preferences")
        modal: true
        padding: 20
        anchors.centerIn: parent
        width: Math.min(520, window.width - 48)
        standardButtons: Dialog.Close
        ColumnLayout {
            width: parent.width
            spacing: 10
            Label { text: bridge.message("Downloads"); font.bold: true }
            Switch {
                text: bridge.message("Ask Where to Save Each File")
                checked: bridge.asks_where_to_save()
                onToggled: bridge.set_asks_where_to_save(checked)
            }
            Label {
                Layout.fillWidth: true
                wrapMode: Text.Wrap
                color: palette.placeholderText
                text: bridge.message("Otherwise files go straight to the download folder")
            }
            RowLayout {
                Layout.fillWidth: true
                Label { text: bridge.message("Download Folder"); Layout.fillWidth: true }
                Button {
                    text: bridge.download_folder().split("/").pop()
                    onClicked: folderDialog.open()
                }
            }
        }
    }

    FolderDialog {
        id: folderDialog
        title: bridge.message("Choose Folder")
        onAccepted: bridge.set_download_folder(decodeURIComponent(selectedFolder.toString().replace("file://", "")))
    }

    Popup {
        id: toastPopup
        x: (window.width - width) / 2
        y: window.height - height - 24
        padding: 12
        closePolicy: Popup.NoAutoClose
        background: Rectangle { radius: 8; color: "#323232" }
        Label { id: toastLabel; color: "white"; textFormat: Text.PlainText }
    }
    Timer { id: toastTimer; interval: 4000; onTriggered: toastPopup.close() }

    Component.onCompleted: {
        const typed = Qt.application.arguments.length > 1 ? bridge.destination(Qt.application.arguments[1]) : ""
        window.openTab(typed)
    }
}
