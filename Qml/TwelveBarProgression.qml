import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15 as QQC2
import org.kde.kirigami 2.18 as Kirigami
import QtQuick.Layouts 1.12

Grid {
    id: twelveBarBluesGrid
    rows: 3
    columns: 4
    spacing: 1

    property int cellWidth: twelveBarBluesGrid.width / 4
    property int cellHeight: twelveBarBluesGrid.height / 3
    property bool playing: false
    property variant rectangles: [r1, r2, r3, r4, r5, r6, r7, r8, r9, r10, r11, r12]
    property int currentRectangle: -1
    property int beatCount: -1

    onPlayingChanged: {
        currentRectangle = -1
        beatCount = -1
    }

    // Try to come up a way to make this more `QML` in the future.
    function advance() {
        beatCount = (beatCount + 1) % 4;
        if (beatCount === 0) {
            if (currentRectangle !== -1) {
                rectangles[currentRectangle].clear()
            }
            currentRectangle = (currentRectangle + 1) % 12
        }
        rectangles[currentRectangle].advance()
        console.log("Beat Count", beatCount, "Current Rect", currentRectangle)
    }

    TwelveBarProgressionRect {
        id: r1
        text: "I"
        width: cellWidth
        height: cellHeight
    }
    TwelveBarProgressionRect {
        id: r2
        text: "I"
        width: cellWidth
        height: cellHeight
    }
    TwelveBarProgressionRect {
        id: r3
        text: "I"
        width: cellWidth
        height: cellHeight
    }
    TwelveBarProgressionRect {
        id: r4
        text: "I"
        width: cellWidth
        height: cellHeight
    }
    TwelveBarProgressionRect {
        id: r5
        text: "IV"
        width: cellWidth
        height: cellHeight
    }
    TwelveBarProgressionRect {
        id: r6
        text: "IV"
        width: cellWidth
        height: cellHeight
    }
    TwelveBarProgressionRect {
        id: r7
        text: "I"
        width: cellWidth
        height: cellHeight
    }
    TwelveBarProgressionRect {
        id: r8
        text: "I"
        width: cellWidth
        height: cellHeight
    }
    TwelveBarProgressionRect {
        id: r9
        text: "V"
        width: cellWidth
        height: cellHeight
    }
    TwelveBarProgressionRect {
        id: r10
        text: "IV"
        width: cellWidth
        height: cellHeight
    }
    TwelveBarProgressionRect {
        id: r11
        text: "I"
        width: cellWidth
        height: cellHeight
    }
    TwelveBarProgressionRect {
        id: r12
        text: "I or IV"
        width: cellWidth
        height: cellHeight
    }
}
