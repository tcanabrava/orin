import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15 as QQC2
import org.kde.kirigami 2.18 as Kirigami

Kirigami.ApplicationWindow {
    width: 640
    height: 480
    visible: true

    // TODO: set this as a configuration.
    property int beatsPerMinute: 1000
    title: qsTr("Hello World")

    pageStack.initialPage: InitialScreen {
        onInstrumentSelected: {
            pageStack.replace("qrc:/Qml/" + instrumentPage, {beatsPerMinute: parent.beatsPerMinute})
        }
    }

}
