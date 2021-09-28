#include "HarmonicaSheet.h"

#include "HarmonicaParser.h"

#include <QDebug>
#include <QFile>
#include <QUrl>

QString HarmonicaSheet::file() const
{
    return QString();
}

void HarmonicaSheet::setFile(const QString& file)
{
    qDebug() << "Set file to" << file;
    m_file = file;

    Q_EMIT fileChanged(m_file);

    QUrl localFile(m_file);

    HarmonicaParser parser;
    parser.setFile(localFile.toLocalFile());
    const bool parsed = parser.parse();

    if (!parsed) {
        setSoundData({});
        setBpm(0);
        qDebug() << "Finalized with" << parser.errorString();
        Q_EMIT errorMessage(tr("Error parsing file"));
        return;
    }

    setSoundData(parser.data());
    setBpm(parser.bpm());
}

int HarmonicaSheet::bpm() const
{
    return m_bpm;
}

void HarmonicaSheet::setBpm(int beatsPerMinute)
{
    m_bpm = beatsPerMinute;
}

int HarmonicaSheet::bpmDelta() const
{
    return m_bpmDelta;
}

void HarmonicaSheet::setBpmDelta(int delta)
{
    m_bpmDelta = delta;
}

void HarmonicaSheet::setSoundData(const std::vector<HarmonicaSoundData>& soundData)
{
    m_soundData = soundData;

    qDebug() << "Sound data set!" << m_soundData.size();
}

// a change in bpm will trigger a recalculate.
void HarmonicaSheet::precalculate()
{

}

// starts emmiting soundData.
void HarmonicaSheet::start()
{

}

void HarmonicaSheet::stop()
{

}
