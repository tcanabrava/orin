import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15 as QQC2
import org.kde.kirigami 2.18 as Kirigami
import QtQuick.Layouts 1.12

// Really don't like the idea of importing labs.
// Test those things in Qt 6.2
import Qt.labs.folderlistmodel 2.15
import Qt.labs.platform 1.1
import QtQuick.Dialogs 1.3 as Dialogs

Kirigami.OverlaySheet {
    id: addSheet
    header: Kirigami.Heading {
        text: qsTr("Partiture List")
    }

    ColumnLayout {
        Dialogs.FileDialog {
            id: folderDialog
            folder: StandardPaths.writableLocation(StandardPaths.DocumentsLocation)
            selectFolder: true
            onAccepted: folderField.text = folderDialog.fileUrl
        }

        RowLayout {
            Layout.fillWidth: true
            QQC2.Label {
                text: qsTr("Folder:")
            }
            QQC2.TextField {
                id: folderField
            }
            QQC2.Button {
                text: qsTr("Find")
                onClicked: {
                    folderDialog.open()
                }
            }
        }

        Kirigami.CardsListView {
            Layout.preferredWidth: 400
            Layout.preferredHeight: 400

            model: FolderListModel {
                id: fileModel
                nameFilters: ["*.tcg"]
                folder: "file://" + folderField.text
            }
            delegate:  Kirigami.AbstractCard {
                contentItem: Kirigami.BasicListItem {
                    label: fileName
                }
            }
        }
    }
}
