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

    property variant holes: [r1, r2, r3, r4, r5, r6, r7, r8, r9, r10]

    function clear() {
        for (let i = 0; i < holes.length; i++) {
            holes[i].color = silentColor;
        }
    }

    function paintHolesSoundData(soundData) {
        clear()
        for (let idx in soundData.holes) {
            // holes starts with idx 1.
            let hole_idx = soundData.holes[idx] - 1
            holes[hole_idx].color = drawColor
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
            holes[index].color = type === "draw" ? drawColor : blowColor
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
        Rectangle { id: r1;  color: silentColor; width: 10; height: 18}
        Rectangle { id: r2;  color: silentColor; width: 10; height: 18}
        Rectangle { id: r3;  color: silentColor; width: 10; height: 18}
        Rectangle { id: r4;  color: silentColor; width: 10; height: 18}
        Rectangle { id: r5;  color: silentColor; width: 10; height: 18}
        Rectangle { id: r6;  color: silentColor; width: 10; height: 18}
        Rectangle { id: r7;  color: silentColor; width: 10; height: 18}
        Rectangle { id: r8;  color: silentColor; width: 10; height: 18}
        Rectangle { id: r9;  color: silentColor; width: 10; height: 18}
        Rectangle { id: r10; color: silentColor; width: 10; height: 18}
    }
}
