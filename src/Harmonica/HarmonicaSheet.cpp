#include "HarmonicaSheet.h"

#include "HarmonicaParser.h"

#include <QDebug>
#include <QFile>
#include <QUrl>

HarmonicaSheet::HarmonicaSheet()
{
    connect(&m_bpmTimer, &QTimer::timeout, this, &HarmonicaSheet::timerTick);
}

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
    m_currIdx = 0;
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
    m_bpmTotal = m_bpm + m_bpmDelta;

    Q_EMIT bpmChanged();
    Q_EMIT bpmTotalChanged();
}

int HarmonicaSheet::bpmDelta() const
{
    return m_bpmDelta;
}

void HarmonicaSheet::setBpmDelta(int delta)
{
    m_bpmDelta = delta;
    m_bpmTotal = m_bpm + m_bpmDelta;

    Q_EMIT bpmDeltaChanged();
    Q_EMIT bpmTotalChanged();
}

void HarmonicaSheet::setSoundData(const std::vector<HarmonicaSoundData>& soundData)
{
    m_soundData = soundData;
    m_totalBeats = (int) soundData.size();
    Q_EMIT totalBeatsChanged(m_totalBeats);
}

HarmonicaSoundData HarmonicaSheet::beatAt(int idx) {
    return m_soundData[idx];
}

// a change in bpm will trigger a recalculate.
void HarmonicaSheet::precalculate()
{

}

// starts emmiting soundData.
void HarmonicaSheet::start()
{
    qreal total = 60 / (qreal) m_bpmTotal * 1000;
    m_bpmTimer.start(total);

    // TODO: There are three timers currently, we need to use only *one* timer
    // to control the music. One is here. other is in TwelveBarProgression and the
    // other one in TwelveBarProgressionRect
    timerTick();
}

void HarmonicaSheet::stop()
{
    m_currIdx = 0;
    m_bpmTimer.stop();
}

void HarmonicaSheet::pause()
{
    m_bpmTimer.stop();
}

void HarmonicaSheet::timerTick() {
    if (m_currIdx < m_soundData.size()) {
        Q_EMIT sendBeat(m_soundData[m_currIdx], m_currIdx);
        m_currIdx += 1;
    } else {
        stop();
    }
}

int HarmonicaSheet::bpmTotal() const {
    return m_bpmTotal;
}

int HarmonicaSheet::totalBeats() const {
    return m_totalBeats;
}

int HarmonicaSheet::currentBeat() const {
    return m_currentBeat;
}
