import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15 as QQC2
import org.kde.kirigami 2.18 as Kirigami
import QtQuick.Layouts 1.12

Grid {
    id: twelveBarBluesGrid
    rows: 3
    columns: 4
    spacing: 0

    property int cellWidth: twelveBarBluesGrid.width / 4
    property int cellHeight: twelveBarBluesGrid.height / 3
    property bool playing: false
    property variant rectangles: [r1, r2, r3, r4, r5, r6, r7, r8, r9, r10, r11, r12]
    property int currentRectangle: -1;
    property int bpm: (1000 * 60) / 85 // beats per minute. defaults to 85

    onPlayingChanged: {
        if (currentRectangle != -1) {
            rectangles[currentRectangle].color = "white"
        }

        currentRectangle = -1
    }

    Timer {
        running: parent.playing
        repeat: true
        interval: bpm
        onTriggered: {
            if (currentRectangle != -1) {
                rectangles[currentRectangle].color = "white"
            }
            currentRectangle = (currentRectangle + 1) % 12
            if (currentRectangle != -1) {
                rectangles[currentRectangle].color = "cyan"
            }
        }
    }

    Rectangle {
        id: r1
        border.color: "black"
        width: cellWidth
        height: cellHeight
    }
    Rectangle {
        id: r2
        border.color: "black"
        width: cellWidth
        height: cellHeight
    }
    Rectangle {
        id: r3
        border.color: "black"
        width: cellWidth
        height: cellHeight
    }
    Rectangle {
        id: r4
        border.color: "black"
        width: cellWidth
        height: cellHeight
    }
    Rectangle {
        id: r5
        border.color: "black"
        width: cellWidth
        height: cellHeight
    }
    Rectangle {
        id: r6
        border.color: "black"
        width: cellWidth
        height: cellHeight
    }
    Rectangle {
        id: r7
        border.color: "black"
        width: cellWidth
        height: cellHeight
    }
    Rectangle {
        id: r8
        border.color: "black"
        width: cellWidth
        height: cellHeight
    }
    Rectangle {
        id: r9
        border.color: "black"
        width: cellWidth
        height: cellHeight
    }
    Rectangle {
        id: r10
        border.color: "black"
        width: cellWidth
        height: cellHeight
    }
    Rectangle {
        id: r11
        border.color: "black"
        width: cellWidth
        height: cellHeight
    }
    Rectangle {
        id: r12
        border.color: "black"
        width: cellWidth
        height: cellHeight
    }
}
