import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// The preferences: the interface language, the toolbar buttons, scrolling and downloads, in rounded groups.
Dialog {
    id: dialog
    required property var bridge
    property string downloadFolder
    property int page: 0

    /// A toolbar button was shown or hidden.
    signal toolbarChanged()
    /// Something worth telling the person happened (a toast).
    signal notice(string text)
    signal folderRequested()

    modal: true
    padding: 0
    anchors.centerIn: parent
    width: Math.min(640, parent.width - 48)
    height: Math.min(720, parent.height - 48)
    background: Rectangle {
        radius: 12
        color: palette.window
        border.width: 1
        border.color: Qt.alpha(palette.windowText, 0.2)
    }

    function say(text) { return bridge.message(text) }

    // The toolbar buttons InDuckGo offers: stored name, title, explanation.
    readonly property var toolbarButtons: [
        { id: "backAndForward", title: "Back and Forward", subtitle: "Go to the previous or next page" },
        { id: "reload", title: "Reload", subtitle: "Load the page again, or stop loading it" },
        { id: "protections", title: "Protection Shield", subtitle: "Switch tracker protection off or on for the site" },
        { id: "burnData", title: "Burn Browsing Data", subtitle: "Delete this profile's cookies, site data and cache" },
        { id: "downloads", title: "Downloads", subtitle: "The five latest downloads, and the download folder" },
        { id: "newTab", title: "New Tab", subtitle: "Open a new tab" }
    ]

    contentItem: ColumnLayout {
        spacing: 0

        Item {
            Layout.fillWidth: true
            Layout.preferredHeight: 62
            Row {
            anchors.centerIn: parent
            spacing: 6
            Repeater {
                model: [
                    { title: "Interface", icon: "locale" },
                    { title: "Appearance", icon: "appearance" },
                    { title: "Downloads", icon: "download" }
                ]
                delegate: AbstractButton {
                    required property var modelData
                    required property int index
                    checked: dialog.page === index
                    onClicked: dialog.page = index
                    implicitHeight: 34
                    implicitWidth: pageLabel.implicitWidth + 56
                    background: Rectangle {
                        radius: 9
                        color: parent.checked ? Qt.alpha(palette.windowText, 0.1)
                             : parent.hovered ? Qt.alpha(palette.windowText, 0.05) : "transparent"
                    }
                    contentItem: Item {
                        Row {
                            spacing: 8
                            anchors.centerIn: parent
                            TintedIcon { anchors.verticalCenter: parent.verticalCenter; name: modelData.icon; size: 16 }
                            Label { id: pageLabel; anchors.verticalCenter: parent.verticalCenter; text: dialog.say(modelData.title); font.bold: true }
                        }
                    }
                }
            }
            }
            AbstractButton {
                anchors.right: parent.right
                anchors.rightMargin: 14
                anchors.verticalCenter: parent.verticalCenter
                implicitWidth: 24
                implicitHeight: 24
                onClicked: dialog.close()
                background: Rectangle {
                    radius: 12
                    color: Qt.alpha(palette.windowText, parent.hovered ? 0.14 : 0.08)
                }
                contentItem: Item { TintedIcon { anchors.centerIn: parent; name: "close"; size: 16 } }
            }
        }

        StackLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: dialog.page

            ScrollView {
                id: interfacePage
                contentWidth: availableWidth
                clip: true
                ColumnLayout {
                    width: interfacePage.availableWidth
                    spacing: 18
                    Item { Layout.preferredHeight: 6 }
                    PreferencesGroup {
                        Layout.leftMargin: 54
                        Layout.rightMargin: 54
                        title: dialog.say("Language")
                        description: dialog.say("The language of menus, dialogs and messages, in every profile and web app.")
                        PreferencesRow {
                            title: dialog.say("Language")
                            subtitle: dialog.say("Applies the next time InDuckGo starts")
                            ComboBox {
                                id: languageBox
                                Layout.preferredWidth: 200
                                background: null
                                padding: 0
                                contentItem: Label {
                                    text: languageBox.displayText
                                    elide: Text.ElideRight
                                    horizontalAlignment: Text.AlignRight
                                    verticalAlignment: Text.AlignVCenter
                                    rightPadding: 26
                                }
                                indicator: TintedIcon {
                                    x: languageBox.width - width
                                    anchors.verticalCenter: parent.verticalCenter
                                    name: "down"
                                }
                                model: JSON.parse(dialog.bridge.language_names_json())
                                currentIndex: dialog.bridge.language_position()
                                onActivated: function(position) {
                                    if (dialog.bridge.set_language_position(position))
                                        dialog.notice(dialog.say("The language changes when InDuckGo is restarted"))
                                }
                            }
                        }
                    }
                    Item { Layout.preferredHeight: 6 }
                }
            }

            ScrollView {
                id: appearancePage
                contentWidth: availableWidth
                clip: true
                ColumnLayout {
                    width: appearancePage.availableWidth
                    spacing: 18
                    Item { Layout.preferredHeight: 6 }
                    PreferencesGroup {
                        Layout.leftMargin: 54
                        Layout.rightMargin: 54
                        title: dialog.say("Toolbar Buttons")
                        description: dialog.say("Buttons shown around the address bar in this profile's windows.")
                        Repeater {
                            model: dialog.toolbarButtons
                            delegate: PreferencesRow {
                                required property var modelData
                                title: dialog.say(modelData.title)
                                subtitle: dialog.say(modelData.subtitle)
                                AdwaitaSwitch {
                                    checked: dialog.bridge.toolbar_shows(modelData.id)
                                    onToggled: {
                                        dialog.bridge.set_toolbar_shown(modelData.id, checked)
                                        dialog.toolbarChanged()
                                    }
                                }
                            }
                        }
                    }
                    PreferencesGroup {
                        Layout.leftMargin: 54
                        Layout.rightMargin: 54
                        title: dialog.say("Scrolling")
                        PreferencesRow {
                            title: dialog.say("Smooth mouse wheel scrolling")
                            subtitle: dialog.say("Animates every step of the mouse wheel. Pages that are already open need a reload.")
                            AdwaitaSwitch {
                                checked: dialog.bridge.smooth_wheel()
                                onToggled: dialog.bridge.set_smooth_wheel(checked)
                            }
                        }
                    }
                    Item { Layout.preferredHeight: 6 }
                }
            }

            ScrollView {
                id: downloadsPage
                contentWidth: availableWidth
                clip: true
                ColumnLayout {
                    width: downloadsPage.availableWidth
                    spacing: 18
                    Item { Layout.preferredHeight: 6 }
                    PreferencesGroup {
                        Layout.leftMargin: 54
                        Layout.rightMargin: 54
                        title: dialog.say("Downloads")
                        PreferencesRow {
                            title: dialog.say("Ask Where to Save Each File")
                            subtitle: dialog.say("Otherwise files go straight to the download folder")
                            AdwaitaSwitch {
                                checked: dialog.bridge.asks_where_to_save()
                                onToggled: dialog.bridge.set_asks_where_to_save(checked)
                            }
                        }
                        PreferencesRow {
                            title: dialog.say("Download Folder")
                            subtitle: dialog.downloadFolder
                            IconButton {
                                iconName: "folder"
                                tip: dialog.say("Choose Folder")
                                onClicked: dialog.folderRequested()
                            }
                        }
                    }
                    Item { Layout.preferredHeight: 6 }
                }
            }
        }
    }
}
