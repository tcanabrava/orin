import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15 as QQC2
import org.kde.kirigami 2.18 as Kirigami
import QtQuick.Layouts 1.12

Rectangle {
    id: root
    property color silentColor: "transparent"
    property color blowColor: "red"
    property color drawColor: "green"

    function clear() {
        for (let i = 0; i < holes.count; i++) {
            holes.itemAt(i).color = silentColor;
        }
    }

    function paintHolesSoundData(soundData) {
        clear()
        for (let idx in soundData.holes) {
            // holes starts with idx 1.
            let hole_idx = soundData.holes[idx] - 1
            holes.itemAt(hole_idx).color = drawColor
        }
    }

    function paintHoles(text, type) {
        clear()

        let keys = text.split(';')
        if (keys.length === 0) {
            return;
        }

        for (let hole in keys) {
            let index = parseInt(keys[hole])
            if (Number.isNaN(index)) {
                return
            }
            holes.itemAt(index).color = type === "draw" ? drawColor : blowColor
        }
    }

    Image {
        id: svg
        source: "qrc:/Images/harmonica_bg.svg"
        fillMode: Image.PreserveAspectFit

        width: 300
        x: 0
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
    }

    // Those rectangles represents the notes being played.
    RowLayout {
        anchors.horizontalCenter: parent.horizontalCenter
        y: 29

        spacing: 13
        Repeater {
            id: holes
            model: 10
            Rectangle { color: silentColor; width: 10; height: 18}
        }
    }
}
