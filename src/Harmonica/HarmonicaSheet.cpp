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
    m_file = file;

    Q_EMIT fileChanged(m_file);

    QUrl localFile(m_file);

    HarmonicaParser parser;
    parser.setFile(localFile.toLocalFile());
    const bool parsed = parser.parse();

    if (!parsed) {
        setSoundData({});
        setBpm(0);
        Q_EMIT errorMessage(tr("Error parsing file"));
        setReady(false);
        return;
    }

    setSoundData(parser.data());
    setBpm(parser.bpm());
    setReady(true);
}

void HarmonicaSheet::setReady(bool r)
{
    m_ready = r;
    Q_EMIT readyChanged(r);
}

bool HarmonicaSheet::ready() const
{
    return m_ready;
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
}

// a change in bpm will trigger a recalculate.
void HarmonicaSheet::precalculate()
{

}

bool HarmonicaSheet::running() const
{
    return m_running;
}

void HarmonicaSheet::setRunning(bool r)
{
    if (m_running == r) {
        return;
    }

    m_running = r;
    Q_EMIT runningChanged(r);

    if (m_running) {
        start();
    } else {
        stop();
    }
}

// starts emmiting soundData.
void HarmonicaSheet::start()
{
    qDebug() << "Start called";
}

void HarmonicaSheet::stop()
{
    qDebug() << "Stop called";
}
