import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15 as QQC2
import org.kde.kirigami 2.18 as Kirigami
import QtQuick.Layouts 1.12

import orin.music.harmonica 1.0 as Orin

Rectangle {
    id: root

    border.color: "blue"
    color: "transparent"
    width: parent.width
    height: childrenRect.height

    property QtObject sheet
    property int spacing: 13

    property variant notes : []
    function createSpriteObject(row, column, soundType) {
        let component = Qt.createComponent("BluesHarmonicaNoteBlow.qml");
        let sprite = component.createObject(root, {direction: soundType});
        let verticalSpacing = row * spacing + row * sprite.height;
        let horizontalSpacing = column * spacing + column * sprite.width;

        notes.push(sprite)
        sprite.x = horizontalSpacing
        sprite.y = verticalSpacing
    }

    function createSpriteObjects() {
        for (let row = 0; row < sheet.totalBeats; row += 1) {
            let soundData = sheet.beatAt(row);
            for (let hole = 0; hole < soundData.holes.length; hole += 1) {
                createSpriteObject(row, soundData.holes[hole], soundData.soundType)
            }
        }
    }

    function clearSpriteObjects() {
        for (let idx in notes) {
            let child = notes[idx]
            child.destroy()
        }
        notes = []
    }

    Connections {
        target: sheet
        function onReadyChanged() {
            clearSpriteObjects()
            createSpriteObjects()
        }
    }
}
