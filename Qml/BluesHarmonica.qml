import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15 as QQC2
import org.kde.kirigami 2.18 as Kirigami
import QtQuick.Layouts 1.12

import orin.music.harmonica 1.0 as Orin

Kirigami.Page {
    // TODO: Harmonica Visualizer
    property int beatsPerMinute: 6000 / preferences.general.beats_per_minute

    PartitureChooserOverlay {
        id: partitureOverlay
        onPartitureChoosed: {
            harmonicasheet.file = folder + "/" + fileName
            title.text = fileName
            close();
        }
    }

    Orin.HarmonicaSheet {
        id: harmonicasheet
        running: controlBar.playing
    }

    RowLayout {
        anchors.fill: parent
        id: mainLayout
        ColumnLayout {
            Text {
                id: title
                Layout.fillWidth: true
            }
            BluesHarmonicaChordProgression {
                Layout.fillHeight: true
                Layout.preferredWidth: mainLayout.width * 0.5
                enabled: harmonicasheet.ready
            }

            TwelveBarProgression {
                Layout.preferredHeight: mainLayout.height * 0.33
                Layout.preferredWidth: mainLayout.width * 0.5
                playing: controlBar.playing
                enabled: harmonicasheet.ready
            }
        }

        ColumnLayout {
            Rectangle {
                border.color: "blue"
                Layout.preferredWidth: mainLayout.width * 0.5
                Layout.fillHeight: true
            }
            ControlBar {
                id: controlBar
                ready: harmonicasheet.ready
                Layout.fillWidth: true
                onRequestPartiture: {
                    partitureOverlay.open()
                }
            }
        }
    }
}
