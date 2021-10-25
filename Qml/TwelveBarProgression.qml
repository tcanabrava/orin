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
    property int currentRectangle: -1
    property int beatCount: -1

    property variant texts: [
        "I", "I", "I", "I",
        "IV", "IV", "I", "I",
        "V", "IV", "I", "I or IV"
    ]

    function reset() {
        if (currentRectangle !== -1) {
            repeater.itemAt(currentRectangle).clear()
        }
        currentRectangle = -1
        beatCount = -1
    }

    // Try to come up a way to make this more `QML` in the future.
    function advance() {
        beatCount = (beatCount + 1) % 4;
        if (beatCount === 0) {
            if (currentRectangle !== -1) {
                repeater.itemAt(currentRectangle).clear()
            }
            currentRectangle = (currentRectangle + 1) % 12
        }
        repeater.itemAt(currentRectangle).advance()
        console.log("Beat Count", beatCount, "Current Rect", currentRectangle)
    }

    Repeater {
        id: repeater
        model: 12
        TwelveBarProgressionRect {
            text: twelveBarBluesGrid.texts[index]
            width: cellWidth
            height: cellHeight
        }
    }
}
