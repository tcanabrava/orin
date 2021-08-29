import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15 as QQC2
import QtQuick.Layouts 1.12
import org.kde.kirigami 2.18 as Kirigami

Kirigami.Page {
    property variant instruments: [
        {
            name: qsTr("Flute"),
            icon: "",
            description: "",
            qmlPage: "Flute.qml"
        },
        {
            name: qsTr("Blues Harmonica"),
            icon: "",
            description: "",
            qmlPage: "BluesHarmonica.qml"
        }
    ]

    signal instrumentSelected(string instrument)

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
                delegate: Kirigami.BasicListItem {
                    label: modelData.name
                    icon: modelData.icon
                    onClicked: instrumentSelected(modelData.qmlPage)
                }
            }
        }
    }
}
