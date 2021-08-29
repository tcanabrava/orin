import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15 as QQC2
import org.kde.kirigami 2.18 as Kirigami
import QtQuick.Layouts 1.12

RowLayout {
    property bool playing: false
    property int beatsPerMinute: 1000

    signal start()
    signal stop()

    Item{
        Layout.fillWidth: true
    }
    Timer {

    }
    QQC2.Button {
        text: playing ? qsTr("Stop") : qsTr("Start")
        onClicked: {
            console.log("Bpm", parent.beatsPerMinute)
            playing = !playing
            playing ? stop() : start()
        }
    }

    Item{
        Layout.fillWidth: true
    }
}
