import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15 as QQC2

Window {
    width: 640
    height: 480
    visible: true
    title: qsTr("Hello World")
    QQC2.StackView {
        id: mainStack
        anchors.fill: parent
    }
}

