import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15 as QQC2
import org.kde.kirigami 2.18 as Kirigami
import QtQuick.Layouts 1.12

import orin.music.harmonica 1.0 as Orin

Rectangle {
    id: root
    clip: true

    color: "transparent"
    property QtObject sheet
    property int spacing: 13

    function createSpriteObject(row, column, soundType) {
        let component = Qt.createComponent("BluesHarmonicaNoteBlow.qml");
        let sprite = component.createObject(root, {direction: soundType});
        let verticalSpacing = row * spacing + row * sprite.height;
        let horizontalSpacing = column * spacing + column * sprite.width;
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
        for (let idx in children) {
            children[idx].destroy()
        }
    }

    Connections {
        target: sheet
        function onReadyChanged() {
            clearSpriteObjects()
            createSpriteObjects()
        }
    }
}
