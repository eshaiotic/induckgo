import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtWebEngine
import QtWebChannel
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
    title: isWebApp ? (currentTab && currentTab.title ? currentTab.title : launch.webApp.name)
         : currentTab && currentTab.title ? currentTab.title + " — InDuckGo" : "InDuckGo"

    BrowserBridge { id: bridge }
    WebAppBridge { id: webApps }

    // How this process was launched: the browser, or one installed web app with its own window and profile.
    readonly property var launch: JSON.parse(webApps.launch_json()) || ({ webApp: null, addresses: [] })
    readonly property bool isWebApp: launch.webApp !== null

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

    // Qt's web-channel client followed by Guard's page script (empty when the client is missing).
    readonly property string guardSource: bridge.guard_page_script()

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
    readonly property var currentTab: tabList[currentIndex] || null
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

    function toast(text, actionText, action) {
        toastLabel.text = text
        toastPopup.actionText = actionText || ""
        toastPopup.action = action || null
        toastPopup.open()
        toastTimer.restart()
    }

    function navigate(typed) {
        const destination = bridge.destination(typed)
        if (destination !== "" && currentTab) {
            currentTab.load(destination)
            currentTab.forceActiveFocus()
        }
    }

    // Whether the page on screen can become an app: a web page, in the browser (an app is one already).
    readonly property bool installable: !isWebApp && currentTab !== null && /^https?:\/\//.test(currentTab.address)

    function installAsApp() { installation.offer(currentTab) }

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
        onTriggered: {
            if (downloadsPopover.opened) downloadsPopover.close()
            else if (Date.now() - downloadsPopover.closedAt > 250) downloadsPopover.open()
        }
    }
    Action {
        id: installAppAction
        text: bridge.message("Install Site as App…")
        icon.source: "qrc:/icons/install-app.svg"
        icon.color: window.palette.windowText
        enabled: window.installable
        onTriggered: window.installAsApp()
    }
    Action {
        id: protectSiteAction
        text: bridge.message("Protect This Site")
        icon.source: "qrc:/icons/shield-on.svg"
        icon.color: window.palette.windowText
        enabled: window.currentHost !== ""
        onTriggered: {
            bridge.set_site_protected(window.currentHost, !bridge.is_site_protected(window.currentHost))
            window.currentTab.reload()
        }
    }
    Action {
        id: openInBrowserAction
        text: bridge.message("Open Page in Browser")
        icon.source: "qrc:/icons/open.svg"
        icon.color: window.palette.windowText
        onTriggered: if (window.currentTab && webApps.open_in_browser(window.currentTab.address)) window.close()
    }
    Action {
        id: clearAppDataAction
        text: bridge.message("Clear App Data…")
        icon.source: "qrc:/icons/fire.svg"
        icon.color: window.palette.windowText
        onTriggered: clearAppDataDialog.open()
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
                    visible: window.shows("backAndForward") || window.isWebApp
                    iconName: "back"
                    tip: bridge.message("Back")
                    enabled: window.currentTab && window.currentTab.canGoBack
                    onClicked: window.currentTab.goBack()
                }
                IconButton {
                    visible: window.shows("backAndForward") && !window.isWebApp
                    iconName: "forward"
                    tip: bridge.message("Forward")
                    enabled: window.currentTab && window.currentTab.canGoForward
                    onClicked: window.currentTab.goForward()
                }
                IconButton {
                    visible: window.shows("reload") || window.isWebApp
                    iconName: window.currentTab && window.currentTab.isLoading ? "stop" : "reload"
                    tip: window.currentTab && window.currentTab.isLoading ? bridge.message("Load the page again, or stop loading it") : bridge.message("Reload")
                    onClicked: window.currentTab.isLoading ? window.currentTab.stop() : window.currentTab.reload()
                }
            }

            // A web app has no address bar: its name (the page's title) over the host of the page on screen, the
            // host flagged when the page is not on the app's own site.
            Column {
                id: appTitle
                visible: window.isWebApp
                anchors.centerIn: parent
                width: Math.max(160, rightGroup.x - leftGroup.x - leftGroup.width - 48)
                readonly property bool onOwnSite: !window.currentTab || webApps.is_app_page(window.currentTab.address)
                Label {
                    width: parent.width
                    horizontalAlignment: Text.AlignHCenter
                    elide: Text.ElideRight
                    text: window.title
                    font.bold: true
                    textFormat: Text.PlainText
                }
                Label {
                    width: parent.width
                    horizontalAlignment: Text.AlignHCenter
                    elide: Text.ElideRight
                    text: window.currentHost
                    font.pixelSize: 11
                    color: appTitle.onOwnSite ? Qt.alpha(palette.windowText, 0.6) : (window.isDark ? "#f8e45c" : "#9c6e03")
                    textFormat: Text.PlainText
                    ToolTip.visible: !appTitle.onOwnSite && hoverArea.containsMouse
                    ToolTip.text: window.isWebApp ? bridge.message("This page is on another site, not part of {name}").replace("{name}", window.launch.webApp.name) : ""
                    MouseArea { id: hoverArea; anchors.fill: parent; hoverEnabled: true; acceptedButtons: Qt.NoButton }
                }
            }

            RowLayout {
                id: addressGroup
                anchors.verticalCenter: parent.verticalCenter
                visible: !window.isWebApp
                // Wide: the address takes all the room the two groups of buttons leave. Short: a centred bar of at most
                // 550 px that never reaches either group.
                readonly property bool wide: window.toolbarVersion >= 0 && bridge.wide_address_bar()
                readonly property real room: Math.max(160, rightGroup.x - 12 - (leftGroup.x + leftGroup.width + 12))
                width: wide ? room : Math.min(550, room)
                x: wide ? leftGroup.x + leftGroup.width + 12
                        : Math.max(leftGroup.x + leftGroup.width + 12, (parent.width - width) / 2)
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
                        // The icon at the end of the address bar: this page as an app of its own.
                        AbstractButton {
                            id: installButton
                            readonly property bool offered: window.shows("installApp") && window.installable
                            visible: offered
                            Layout.preferredWidth: 26
                            Layout.preferredHeight: 26
                            onClicked: window.installAsApp()
                            Accessible.name: bridge.message("Install Site as App")
                            ToolTip.visible: hovered
                            ToolTip.delay: 500
                            ToolTip.text: bridge.message("Install Site as App")
                            background: Rectangle {
                                radius: 7
                                color: installButton.down ? Qt.alpha(palette.windowText, 0.16)
                                     : installButton.hovered ? Qt.alpha(palette.windowText, 0.1) : "transparent"
                            }
                            contentItem: Item { TintedIcon { anchors.centerIn: parent; name: "install-app"; size: 16 } }
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
                    visible: window.shows("protections") && !window.isWebApp
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
                    visible: window.shows("burnData") && !window.isWebApp
                    iconName: "fire"
                    tip: bridge.message("Burn Browsing Data")
                    onClicked: burnAction.trigger()
                }
                IconButton {
                    visible: window.shows("newTab") && !window.isWebApp
                    iconName: "new-tab"
                    tip: bridge.message("New Tab")
                    onClicked: newTab.trigger()
                }
                IconButton {
                    id: downloadsButton
                    checked: downloadsPopover.opened
                    tipHidden: downloadsPopover.opened
                    visible: window.shows("downloads") && !window.isWebApp
                    iconName: "download"
                    tip: bridge.message("Downloads")
                    onClicked: showDownloads.trigger()
                }
                IconButton {
                    iconName: "menu"
                    tip: bridge.message("Main Menu")
                    checked: mainMenu.opened
                    tipHidden: mainMenu.opened
                    onClicked: {
                        if (mainMenu.opened) mainMenu.close()
                        else if (Date.now() - mainMenu.closedAt > 250) mainMenu.open()
                    }

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
                        installApp: installAppAction
                        protectSite: protectSiteAction
                        openInBrowser: openInBrowserAction
                        clearAppData: clearAppDataAction
                        webApp: window.isWebApp
                        installable: window.installable
                    }
                }

                Item { Layout.preferredWidth: 1 }
                WindowControls { window: window }
            }
        }

        TabStrip {
            Layout.fillWidth: true
            visible: window.tabList.length > 1 && !window.isWebApp
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
            guardSource: window.guardSource
            appBridge: webApps
            keepsToSite: window.isWebApp
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
        onFinished: function(name, path) {
            window.toast(bridge.message("Downloaded {name}").replace("{name}", name),
                         bridge.message("Show in Folder"), function() { downloads.showInFolder(path) })
        }
        onFailed: function(message) { window.toast(message) }
    }
    DownloadsManager { id: downloadsManager; hub: downloads; bridge: bridge }

    WebAppInstallation {
        id: installation
        bridge: bridge
        webApps: webApps
        onNotice: function(text, actionText, action) { window.toast(text, actionText, action) }
        onInstalled: function(id, name, installedFrom) {
            const message = bridge.message("Installed {name}").replace("{name}", name)
            if (webApps.start(id)) {
                window.handOver(id, installedFrom)
                window.toast(message)
            } else {
                window.toast(message, bridge.message("Open"), function() { webApps.start(id) })
            }
        }
    }

    // Once the app runs, the browser lets go of the site: the tabs that showed it close.
    function handOver(id, installedFrom) {
        const leaving = tabList.filter(function(tab) { return tab === installedFrom || webApps.owns_page(id, tab.address) })
        if (leaving.length === tabList.length) openTab("")
        leaving.forEach(function(tab) { closeTab(tabList.indexOf(tab)) })
    }

    ConfirmDialog {
        id: clearAppDataDialog
        bridge: bridge
        heading: window.isWebApp ? bridge.message("Clear Data of {name}?").replace("{name}", window.launch.webApp.name) : ""
        body: bridge.message("Signs you out and deletes this app's cookies, site data and cache. Nothing else is affected.")
        confirmLabel: "C_lear"
        onConfirmed: window.burn()
    }

    ConfirmDialog {
        id: burnDialog
        bridge: bridge
        heading: bridge.message("Burn Browsing Data?")
        body: bridge.message("Closes all tabs and windows of this profile and deletes its cookies, site data, cache and history.")
        confirmLabel: "_Burn"
        onConfirmed: window.burn()
    }

    AboutDialog {
        id: aboutDialog
        bridge: bridge
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

    DownloadsPopover {
        id: downloadsPopover
        hub: downloads
        bridge: bridge
        onManagerRequested: downloadsManager.open()
    }

    // A short message at the window's foot, dark in both colour schemes, with the action that goes with it.
    Popup {
        id: toastPopup
        property string actionText
        property var action: null
        parent: Overlay.overlay
        x: (parent.width - width) / 2
        y: parent.height - height - 24
        padding: 6
        leftPadding: 16
        closePolicy: Popup.NoAutoClose
        enter: Transition { NumberAnimation { property: "opacity"; from: 0; to: 1; duration: 140 } }
        exit: Transition { NumberAnimation { property: "opacity"; to: 0; duration: 120 } }
        background: Rectangle { radius: height / 2; color: "#3d3d41" }
        contentItem: RowLayout {
            spacing: 8
            Label {
                id: toastLabel
                Layout.maximumWidth: Math.max(160, window.width - 240)
                color: "white"
                font.bold: true
                elide: Text.ElideMiddle
                textFormat: Text.PlainText
            }
            AbstractButton {
                visible: toastPopup.actionText !== ""
                implicitHeight: 34
                implicitWidth: actionLabel.implicitWidth + 32
                onClicked: { if (toastPopup.action) toastPopup.action(); toastPopup.close() }
                background: Rectangle { radius: 17; color: parent.down ? "#6a6a6e" : parent.hovered ? "#5c5c60" : "#4e4e52" }
                contentItem: Label { id: actionLabel; text: toastPopup.actionText; color: "white"; font.bold: true; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter }
            }
            AbstractButton {
                implicitWidth: 30
                implicitHeight: 34
                onClicked: toastPopup.close()
                Accessible.name: "×"
                contentItem: Label { text: "×"; color: "white"; font.bold: true; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter }
                background: Item { }
            }
        }
    }
    Timer { id: toastTimer; interval: 4000; onTriggered: toastPopup.close() }

    Component.onCompleted: {
        if (window.isWebApp) { window.openTab(window.launch.webApp.startUrl); return }
        if (window.launch.addresses.length === 0) { window.openTab(""); return }
        window.launch.addresses.forEach(function(typed) { window.openTab(bridge.destination(typed)) })
    }

    // Later launches of this process's identity hand their requests here: open the addresses, or just come forward.
    Timer {
        interval: 400
        running: true
        repeat: true
        onTriggered: {
            for (const addresses of JSON.parse(webApps.take_forwarded_json())) {
                if (!window.isWebApp) addresses.forEach(function(address) { window.openTab(address) })
                window.show()
                window.raise()
                window.requestActivate()
            }
        }
    }
}
