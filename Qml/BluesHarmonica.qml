import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15 as QQC2
import org.kde.kirigami 2.18 as Kirigami
import QtQuick.Layouts 1.12
import QtWebView 1.15

import orin.music.harmonica 1.0 as Orin

Kirigami.Page {
    // TODO: Harmonica Visualizer
    property int beatsPerMinute: 6000 / preferences.general.beats_per_minute
    function loadUrl() {
        if (harmonicasheet.ready) {
            if (btnAbout.checked) {
                webview.url = harmonicasheet.aboutUrl
            } else {
                webview.url = harmonicasheet.lyricsUrl
            }
        }
    }
    PartitureChooserOverlay {
        id: partitureOverlay
        onPartitureChoosed: {
            harmonicasheet.file = folder + "/" + fileName
            title.text = qsTr("Song: ") + fileName
            close();
        }
    }

    Orin.HarmonicaSheet {
        id: harmonicasheet
        onSendBeat: {
            chordProgression.paintHolesSoundData(beat)
            twelveBarProgression.advance()
        }
    }

    ColumnLayout {
        anchors.fill: parent
        id: mainLayout
        Text {
            id: title
            Layout.alignment: Qt.AlignHCenter
        }
        RowLayout {
            ColumnLayout {
                BluesHarmonicaChordProgression {
                    id: chordProgression
                    Layout.fillHeight: true
                    Layout.preferredWidth: mainLayout.width * 0.5

                    HarmonicaNoteFlow {
                        width: 264
                        height: parent.height

                        sheet: harmonicasheet
                        y: 29
                        anchors.horizontalCenter: parent.horizontalCenter
                    }
                }

                TwelveBarProgression {
                    id: twelveBarProgression
                    Layout.preferredHeight: mainLayout.height * 0.33
                    Layout.preferredWidth: mainLayout.width * 0.5
                    enabled: harmonicasheet.ready
                }
            }
            ColumnLayout {
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
                        target: harmonicasheet
                        function onReadyChanged() {
                            loadUrl()
                        }
                    }
                }
            }
        }
        ControlBar {
            id: controlBar
            ready: harmonicasheet.ready
            Layout.fillWidth: true
            onRequestPartiture: partitureOverlay.open()
            onRequestPlay: harmonicasheet.start()
            onRequestPause: harmonicasheet.pause()
            onRequestStop: {
                harmonicasheet.stop()
                chordProgression.clear()
                twelveBarProgression.clear()
            }
        }
    }
}

