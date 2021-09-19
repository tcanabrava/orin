import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15 as QQC2
import org.kde.kirigami 2.18 as Kirigami
import QtQuick.Layouts 1.12

Kirigami.Page {
    // TODO: Harmonica Visualizer
    property int beatsPerMinute: 6000 / preferences.general.beats_per_minute

    PartitureChooserOverlay {
        id: partitureOverlay
        onPartitureChoosed: {
            console.log("Choosed on", folder, fileName)
        }
    }

    RowLayout {
        anchors.fill: parent
        id: mainLayout
        ColumnLayout {
            BluesHarmonicaChordProgression {
                Layout.fillHeight: true
                Layout.preferredWidth: mainLayout.width * 0.5
            }
            TwelveBarProgression {
                Layout.preferredHeight: mainLayout.height * 0.33
                Layout.preferredWidth: mainLayout.width * 0.5
                playing: controlBar.playing
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
                Layout.fillWidth: true
                onRequestPartiture: {
                    partitureOverlay.open()
                }
            }
        }
    }
}
