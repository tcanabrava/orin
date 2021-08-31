#include "HarmonicaSheet.h"

QString HarmonicaSheet::file() const
{
    return QString();
}

void HarmonicaSheet::setFile(const QString& file)
{
    m_file = file;

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
