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
    // The toolbar is the title bar (as the GTK edition's header bar was); the window draws its own rounded top.
    flags: Qt.Window | Qt.FramelessWindowHint
    color: "transparent"
    title: currentTab && currentTab.title ? currentTab.title + " — InDuckGo" : "InDuckGo"

    BrowserBridge { id: bridge }

    readonly property bool isFramed: visibility !== Window.Maximized && visibility !== Window.FullScreen
    readonly property int cornerRadius: isFramed ? 12 : 0
    background: Rectangle {
        color: palette.window
        topLeftRadius: window.cornerRadius
        topRightRadius: window.cornerRadius
        border.width: window.isFramed ? 1 : 0
        border.color: Qt.alpha(palette.windowText, 0.25)
    }

    // Which toolbar buttons the person chose to show; `toolbarVersion` makes the buttons re-read the choice.
    property int toolbarVersion: 0
    function shows(button) { return toolbarVersion >= 0 && bridge.toolbar_shows(button) }
    property string downloadFolder: bridge.download_folder()

    // Controls follow the desktop's colour scheme even where the style's own palette stays light.
    readonly property bool isDark: Qt.styleHints.colorScheme === Qt.Dark
    // libadwaita's colours, so the window looks as the GTK edition did: light #fafafb with 80 % black text,
    // dark #222226 with white text; cards, popovers and fields are lighter than the window.
    palette.window: isDark ? "#222226" : "#fafafb"
    palette.windowText: isDark ? "#ffffff" : "#2e2e32"
    palette.base: isDark ? "#343438" : "#ffffff"
    palette.text: isDark ? "#ffffff" : "#2e2e32"
    palette.button: isDark ? "#3d3d41" : "#ebebed"
    palette.buttonText: isDark ? "#ffffff" : "#2e2e32"
    palette.placeholderText: isDark ? Qt.rgba(1, 1, 1, 0.55) : Qt.rgba(0, 0, 0, 0.55)
    palette.mid: isDark ? "#666666" : "#a0a0a0"
    palette.highlight: isDark ? "#78aeed" : "#3584e4"
    palette.highlightedText: isDark ? "#1d1d20" : "#ffffff"

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

    // Another window of the same profile: it shares the process, the protections and the downloads.
    function openWindow() {
        const created = Qt.createComponent("qrc:/qt/qml/io/induckgo/browser/qml/Main.qml")
        if (created.status === Component.Ready) created.createObject(null)
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
        icon.source: "qrc:/icons/new-tab.svg"
        icon.color: window.palette.windowText
        shortcut: "Ctrl+T"
        onTriggered: window.openTab("")
    }
    Action {
        id: newWindow
        text: bridge.message("New Window")
        icon.source: "qrc:/icons/window.svg"
        shortcut: "Ctrl+N"
        onTriggered: window.openWindow()
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
        icon.color: window.palette.windowText
        shortcut: "Ctrl+J"
        onTriggered: downloadsPopover.opened ? downloadsPopover.close() : downloadsPopover.open()
    }
    Action {
        id: findInPage
        text: bridge.message("Find in Page…")
        icon.source: "qrc:/icons/search.svg"
        icon.color: window.palette.windowText
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
        icon.color: window.palette.windowText
        shortcut: "Ctrl+Shift+Delete"
        onTriggered: burnDialog.open()
    }
    Action {
        id: aboutAction
        text: bridge.message("About InDuckGo")
        icon.source: "qrc:/icons/info.svg"
        icon.color: window.palette.windowText
        onTriggered: aboutDialog.open()
    }
    Action {
        id: preferencesAction
        text: bridge.message("Preferences…")
        icon.source: "qrc:/icons/settings.svg"
        icon.color: window.palette.windowText
        shortcut: "Ctrl+,"
        onTriggered: preferencesDialog.open()
    }

    header: ColumnLayout {
        spacing: 0

        ToolBar {
            id: headerBar
            Layout.fillWidth: true
            Layout.preferredHeight: 50
            padding: 0
            background: Rectangle {
                color: palette.window
                topLeftRadius: window.cornerRadius
                topRightRadius: window.cornerRadius
                // Empty parts of the bar move the window; a double click maximises it, as a title bar does.
                DragHandler { target: null; onActiveChanged: if (active) window.startSystemMove() }
                TapHandler {
                    onDoubleTapped: window.visibility === Window.Maximized ? window.showNormal() : window.showMaximized()
                }
            }

            // Navigation on the left, the page's tools and the window buttons on the right, the address in the
            // middle of the window whatever the two sides hold (as a GNOME header bar centres its title).
            RowLayout {
                id: leftGroup
                anchors.left: parent.left
                anchors.leftMargin: 6
                anchors.verticalCenter: parent.verticalCenter
                spacing: 6

                IconButton {
                    visible: window.shows("backAndForward")
                    iconName: "back"
                    tip: bridge.message("Back")
                    enabled: window.currentTab && window.currentTab.canGoBack
                    onClicked: window.currentTab.goBack()
                }
                IconButton {
                    visible: window.shows("backAndForward")
                    iconName: "forward"
                    tip: bridge.message("Forward")
                    enabled: window.currentTab && window.currentTab.canGoForward
                    onClicked: window.currentTab.goForward()
                }
                IconButton {
                    visible: window.shows("reload")
                    iconName: window.currentTab && window.currentTab.isLoading ? "stop" : "reload"
                    tip: window.currentTab && window.currentTab.isLoading ? bridge.message("Load the page again, or stop loading it") : bridge.message("Reload")
                    onClicked: window.currentTab.isLoading ? window.currentTab.stop() : window.currentTab.reload()
                }
            }

            RowLayout {
                id: addressGroup
                anchors.verticalCenter: parent.verticalCenter
                x: Math.max(leftGroup.x + leftGroup.width + 12, (parent.width - width) / 2)
                width: Math.min(550, parent.width - 2 * Math.max(leftGroup.width + 6, rightGroup.width + 6) - 24)
                spacing: 6

                Rectangle {
                    id: addressField
                    Layout.fillWidth: true
                    Layout.preferredHeight: 34
                    radius: 9
                    color: addressBar.activeFocus ? palette.base : Qt.alpha(palette.windowText, 0.08)
                    border.width: addressBar.activeFocus ? 2 : 0
                    border.color: palette.highlight
                    clip: true

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 10
                        anchors.rightMargin: 8
                        spacing: 6

                        TintedIcon {
                            readonly property string scheme: window.currentTab ? window.currentTab.address.split(":")[0] : ""
                            Layout.preferredWidth: 16
                            Layout.preferredHeight: 16
                            visible: scheme === "https" || scheme === "http"
                            name: scheme === "https" ? "lock" : "warning"
                            opacity: 0.6
                        }
                        TextField {
                            id: addressBar
                            Layout.fillWidth: true
                            background: null
                            padding: 0
                            selectByMouse: true
                            placeholderText: bridge.message("Search {engine} or enter address").replace("{engine}", "DuckDuckGo")
                            text: window.currentTab ? window.currentTab.address : ""
                            onAccepted: window.navigate(text)
                            onActiveFocusChanged: if (activeFocus) selectAll()
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
                Button {
                    id: zoomBadge
                    visible: window.currentZoom !== 100 && window.zoomVersion >= 0
                    flat: true
                    text: window.currentZoom + "%"
                    font.bold: true
                    implicitHeight: 34
                    implicitWidth: 68
                    background: Rectangle { radius: 9; color: Qt.alpha(palette.windowText, zoomBadge.hovered ? 0.14 : 0.08) }
                    ToolTip.visible: hovered
                    ToolTip.text: bridge.message("Reset Zoom")
                    onClicked: window.changeZoom(0)
                }
            }

            RowLayout {
                id: rightGroup
                anchors.right: parent.right
                anchors.rightMargin: 12
                anchors.verticalCenter: parent.verticalCenter
                spacing: 6

                IconButton {
                    id: shieldButton
                    visible: window.shows("protections")
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
                    visible: window.shows("burnData")
                    iconName: "fire"
                    tip: bridge.message("Burn Browsing Data")
                    onClicked: burnAction.trigger()
                }
                IconButton {
                    visible: window.shows("newTab")
                    iconName: "new-tab"
                    tip: bridge.message("New Tab")
                    onClicked: newTab.trigger()
                }
                IconButton {
                    id: downloadsButton
                    visible: window.shows("downloads")
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
                    onClicked: mainMenu.opened ? mainMenu.close() : mainMenu.open()

                    MainMenu {
                        id: mainMenu
                        window: window
                        bridge: bridge
                        newTab: newTab
                        newWindow: newWindow
                        showDownloads: showDownloads
                        findInPage: findInPage
                        preferences: preferencesAction
                        about: aboutAction
                    }
                }

                Item { Layout.preferredWidth: 1 }
                WindowControls { window: window }
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

    // The frameless window is resized from its edges and corners.
    component ResizeEdge: MouseArea {
        required property int edges
        visible: window.isFramed
        z: 1000
        hoverEnabled: true
        cursorShape: edges === Qt.LeftEdge || edges === Qt.RightEdge ? Qt.SizeHorCursor
                   : edges === Qt.TopEdge || edges === Qt.BottomEdge ? Qt.SizeVerCursor
                   : edges === (Qt.TopEdge | Qt.LeftEdge) || edges === (Qt.BottomEdge | Qt.RightEdge) ? Qt.SizeFDiagCursor
                   : Qt.SizeBDiagCursor
        onPressed: window.startSystemResize(edges)
    }
    ResizeEdge { edges: Qt.LeftEdge; anchors { left: parent.left; top: parent.top; bottom: parent.bottom; topMargin: 8; bottomMargin: 8 } width: 5 }
    ResizeEdge { edges: Qt.RightEdge; anchors { right: parent.right; top: parent.top; bottom: parent.bottom; topMargin: 8; bottomMargin: 8 } width: 5 }
    ResizeEdge { edges: Qt.TopEdge; anchors { top: parent.top; left: parent.left; right: parent.right; leftMargin: 8; rightMargin: 8 } height: 5 }
    ResizeEdge { edges: Qt.BottomEdge; anchors { bottom: parent.bottom; left: parent.left; right: parent.right; leftMargin: 8; rightMargin: 8 } height: 5 }
    ResizeEdge { edges: Qt.TopEdge | Qt.LeftEdge; anchors { top: parent.top; left: parent.left } width: 8; height: 8 }
    ResizeEdge { edges: Qt.TopEdge | Qt.RightEdge; anchors { top: parent.top; right: parent.right } width: 8; height: 8 }
    ResizeEdge { edges: Qt.BottomEdge | Qt.LeftEdge; anchors { bottom: parent.bottom; left: parent.left } width: 8; height: 8 }
    ResizeEdge { edges: Qt.BottomEdge | Qt.RightEdge; anchors { bottom: parent.bottom; right: parent.right } width: 8; height: 8 }

    Component {
        id: tabComponent
        BrowserTab {
            smoothWheelSource: bridge.smooth_wheel_script()
            extensionPromptSource: bridge.duckduckgo_extension_prompt_script()
        }
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

    PreferencesDialog {
        id: preferencesDialog
        bridge: bridge
        downloadFolder: window.downloadFolder
        onToolbarChanged: window.toolbarVersion++
        onNotice: function(text) { window.toast(text) }
        onFolderRequested: folderDialog.open()
    }

    FolderDialog {
        id: folderDialog
        title: bridge.message("Choose Folder")
        onAccepted: {
            bridge.set_download_folder(decodeURIComponent(selectedFolder.toString().replace("file://", "")))
            window.downloadFolder = bridge.download_folder()
        }
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
