#include <QApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QtWebEngine>

#include "Harmonica/HarmonicaSheet.h"

#include "preferences.h"

int main(int argc, char *argv[])
{
#if QT_VERSION < QT_VERSION_CHECK(6, 0, 0)
    QCoreApplication::setAttribute(Qt::AA_EnableHighDpiScaling);
#endif
    QtWebEngine::initialize();

    QApplication app(argc, argv);
    app.setOrganizationName("Tomaz");
    app.setOrganizationDomain("Canabrava");


    qmlRegisterType<HarmonicaSheet>("orin.music.harmonica", 1,0, "HarmonicaSheet");
    qRegisterMetaType<HarmonicaSoundData>("HarmonicaSoundData");

    QQmlApplicationEngine engine;

    engine.rootContext()->setContextProperty("preferences", Preferences::self());

    const QUrl url(QStringLiteral("qrc:/Qml/main.qml"));
    QObject::connect(&engine, &QQmlApplicationEngine::objectCreated,
                     &app, [url](QObject *obj, const QUrl &objUrl) {
        if (!obj && url == objUrl)
            QCoreApplication::exit(-1);
    }, Qt::QueuedConnection);
    engine.load(url);
    int retValue = app.exec();

    Preferences::self()->sync();

    return retValue;
}
