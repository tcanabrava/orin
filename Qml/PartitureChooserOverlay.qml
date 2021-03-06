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

    signal partitureChoosed(string folder, string fileName)

    header: Kirigami.Heading {
        text: qsTr("Partiture List")
    }

    ColumnLayout {
        Dialogs.FileDialog {
            id: folderDialog
            folder: preferences.harmonica.partiture_folder
            selectFolder: true
            onAccepted: folderField.text = folderDialog.fileUrl
        }

        RowLayout {
            Layout.fillWidth: true
            QQC2.Label {
                text: qsTr("Folder:")
            }
            QQC2.TextField {
                Layout.fillWidth: true
                id: folderField
                text: preferences.harmonica.partiture_folder
                onTextChanged: preferences.harmonica.partiture_folder = text
            }
            QQC2.Button {
                text: qsTr("Find")
                onClicked: {
                    folderDialog.open()
                }
            }
        }

        Kirigami.CardsListView {
            Layout.fillWidth: true
            Layout.preferredHeight: 400

            model: FolderListModel {
                id: fileModel
                nameFilters: ["*.tcg"]
                showDirs: false
                folder: folderField.text
            }
            delegate:  Kirigami.AbstractCard {
                contentItem: Kirigami.BasicListItem {
                    label: fileName
                    onClicked: partitureChoosed(folderField.text, fileName)
                }
            }
        }
    }
}
