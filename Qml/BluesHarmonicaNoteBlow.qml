import QtQuick 2.15
import QtQuick.Shapes 1.15
import orin.music.harmonica 1.0 as Orin

Item {
    id: root
    readonly property int baseHeight: 18

    property bool tremmolo: false
    property bool bend: false
    property bool wawah: false
    property int direction: Orin.HarmonicaSoundData.NONE
    property int beatCount: 1

    width: 10
    height: baseHeight * beatCount

    // This allows the shapes to be anti-aliased.
    layer.enabled: true
    layer.samples: 4

    Shape {
        id: drawShape
        visible: direction === Orin.HarmonicaSoundData.DRAW
        anchors.fill: parent
        ShapePath {
            fillColor: "blue"
            strokeWidth: 0
            strokeColor: "blue"
            startX: 0; startY: 0
            PathLine { x: root.width/2; y: root.height }
            PathLine { x: root.width; y: 0 }
            PathLine { x: 0; y: 0 }
        }
    }

    Shape {
        id: blowShape
        visible: direction === Orin.HarmonicaSoundData.BLOW
        anchors.fill: parent
        ShapePath {
            fillColor: "red"
            strokeWidth: 0
            strokeColor: "red"
            startX: 0; startY: root.height
            PathLine { x: root.width/2; y: 0 }
            PathLine { x: root.width; y: root.height }
            PathLine { x: 0; y: root.height }
        }
    }
}
