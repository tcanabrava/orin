import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15 as QQC2
import QtQuick.Layouts 1.12

Item {
    property variant instruments: [qsTr("Flute"), qsTr("Blues Harmonica")]
    anchors.fill: parent

    ColumnLayout {
        anchors.centerIn: parent
        width: 300
        height: 200

        Rectangle {
            border.color: "black"
            border.width: 1
            Layout.fillHeight: true
            Layout.fillWidth: true

            ListView {
                id: instrumentView
                anchors.fill: parent
                anchors.margins: 1

                // TODO: Load the model via plugins.
                model: instruments
                delegate: MouseArea {
                    width: internalData.width
                    height: internalData.height

                    RowLayout  {
                        id: internalData
                        Text {
                            text: modelData
                        }
                    }
                    onClicked: {
                        instrumentView.currentIndex = index
                    }
                }
            }
        }
    }
}
