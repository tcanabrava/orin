import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15 as QQC2
import org.kde.kirigami 2.18 as Kirigami
import QtQuick.Layouts 1.12

Rectangle {
    id: rect
    property variant beats: [b1, b2, b3, b4]
    property int currentBeat: -1
    property double quadWidth: width / 4
    property color innerRectInactive: "transparent"
    property color innerRectActive: "#880088FF"
    property alias text: internalText.text

    state: "inactive"

    function clear() {
        rect.state = "inactive";
        beats[currentBeat].color = innerRectInactive
        currentBeat = -1
    }

    function advance() {
        rect.state = "active";
        if (currentBeat !== -1) {
            beats[currentBeat].color = innerRectInactive;
        }
        currentBeat = (currentBeat + 1) % 4
        beats[currentBeat].color = innerRectActive
    }

    Text {
        id: internalText
        anchors.centerIn: parent
        text: parent.text
    }

    RowLayout {
        anchors.fill: parent
        spacing: 0
        Rectangle {
            id: b1
            color: innerRectInactive
            Layout.preferredWidth: quadWidth
            Layout.fillHeight: true
        }
        Rectangle {
            id: b2
            color: innerRectInactive
            Layout.preferredWidth: quadWidth
            Layout.fillHeight: true
        }
        Rectangle {
            id: b3
            color: innerRectInactive
            Layout.preferredWidth: quadWidth
            Layout.fillHeight: true
        }
        Rectangle {
            id: b4
            color: innerRectInactive
            Layout.preferredWidth: quadWidth
            Layout.fillHeight: true
        }
    }


    states: [
        State {
            name: "inactive"
            PropertyChanges {
                target: rect
                color: "white"
            }
        },
        State {
            name: "active"
            PropertyChanges {
                target: rect
                color: "cyan"
            }
        }
    ]

    transitions: Transition {
        ColorAnimation {
            properties: "color"
            easing.type: Easing.InOutQuad
            duration: 500
        }
    }
}
