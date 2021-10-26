import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15 as QQC2
import org.kde.kirigami 2.18 as Kirigami
import QtQuick.Layouts 1.12
import QtWebView 1.15

import orin.music.harmonica 1.0 as Orin

Kirigami.Page {
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
        onErrorMessage: {
            errorText.text = message
        }
    }

    ColumnLayout {
        anchors.fill: parent
        Text {
            id: title
            Layout.alignment: Qt.AlignHCenter
        }
        Text {
            id: errorText
        }
        QQC2.SplitView {
            orientation: Qt.Horizontal
            Layout.fillWidth: true
            Layout.fillHeight: true
            ColumnLayout {
                id: mainLayout
                QQC2.SplitView.minimumWidth: 400

                BluesHarmonicaChordProgression {
                    id: chordProgression
                    Layout.fillHeight: true
                    Layout.fillWidth: true
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
                    Layout.preferredWidth: mainLayout.width
                    enabled: harmonicasheet.ready
                }
            }
            SheetInformation {
                sheet: harmonicasheet
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

