import QtQuick 2.15
import QtQuick.Controls 2.15 as QQC2
import QtQuick.Layouts 1.12
import QtWebView 1.15

import orin.music.harmonica 1.0 as Orin

ColumnLayout {
    property QtObject sheet

    function loadUrl() {
        webview.url = !harmonicasheet.ready ? ""
                    : btnAbout.checked ? sheet.aboutUrl
                    : sheet.lyricsUrl
    }

    WebView  {
        id: webview
        Layout.fillWidth: true
        Layout.fillHeight: true
    }

    RowLayout {
        QQC2.Button {
            id: btnAbout
            checkable: true
            autoExclusive: true
            text: "About"
            onClicked: {
                loadUrl()
            }
        }
        QQC2.Button {
            id: btnLyrics
            checkable: true
            autoExclusive: true
            checked: true
            text: "Lyrics"
            onClicked: {
                loadUrl()
            }
        }

        Connections {
            target: sheet
            function onReadyChanged() {
                loadUrl()
            }
        }
    }
}
