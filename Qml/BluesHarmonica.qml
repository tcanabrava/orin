import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15 as QQC2
import org.kde.kirigami 2.18 as Kirigami
import QtQuick.Layouts 1.12
Kirigami.Page {
    // TODO: Harmonica Visualizer
    RowLayout {
        anchors.fill: parent
        id: mainLayout
        ColumnLayout {
            Rectangle {
                border.color: "red"
                Layout.fillHeight: true
                Layout.preferredWidth: mainLayout.width * 0.5
            }
            TwelveBarBlues {
                id: bar
                Layout.preferredHeight: mainLayout.height * 0.33
                Layout.preferredWidth: mainLayout.width * 0.5
            }
        }
        Rectangle {
            border.color: "blue"
            Layout.preferredWidth: mainLayout.width * 0.5
            Layout.fillHeight: true
        }
    }
}
