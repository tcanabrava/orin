import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15 as QQC2
import org.kde.kirigami 2.18 as Kirigami
import QtQuick.Layouts 1.12

Rectangle {
    id: rect
    property bool playing: false
    property alias text: internalText.text

    state: "inactive"
    Text {
        id: internalText
        anchors.centerIn: parent
        text: parent.text
    }

    onPlayingChanged: {
        rect.state = rect.state == "inactive" ? "active" : "inactive";
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
