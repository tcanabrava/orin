import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15 as QQC2
import org.kde.kirigami 2.18 as Kirigami
import QtQuick.Layouts 1.12

RowLayout {
    readonly property int playing: 2
    readonly property int pause: 1
    readonly property int stop: 0

    property bool ready: false
    property int currentState: stop

    signal requestPartiture()
    signal requestPlay()
    signal requestPause()
    signal requestStop()

    Item{
        Layout.fillWidth: true
    }

    QQC2.Button {
        visible: ready && currentState !== playing
        text: qsTr("Start")
        onClicked: {
            currentState = playing
            requestPlay()
        }
    }

    QQC2.Button {
        visible: currentState === playing
        text: qsTr("Stop")
        onClicked: {
            currentState = stop
            requestStop()
        }
    }

    QQC2.Button {
        visible: currentState === playing
        text: qsTr("Pause")
        onClicked: {
            currentState = pause
            requestPause()
        }
    }

    QQC2.Button {
        enabled: currentState !== playing
        text: qsTr("Partitures")
        onClicked: requestPartiture()
    }

    Item{
        Layout.fillWidth: true
    }
}
