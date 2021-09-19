#pragma once

#include <QObject>
#include <QString>

#include <vector>

#include "HarmonicaSoundData.h"

/* Defines a harmonica song, completely. */
class HarmonicaSheet: public QObject {
    Q_OBJECT
    Q_PROPERTY(QString file READ file WRITE setFile NOTIFY fileChanged)
    Q_PROPERTY(int bpm READ bpm WRITE setBpm NOTIFY bpmChanged)
    Q_PROPERTY(int bpmDelta READ bpmDelta WRITE setBpmDelta NOTIFY bpmDeltaChanged)
    Q_PROPERTY(int bpmTotal NOTIFY bpmTotalChanged)

private:
    // the speed of the music
    int m_bpm = 0;

    // change the delta if you want something faster or slower.
    int m_bpmDelta = 0;

    // the sum of m_bpm and m_bpmDelta
    int m_bpmTotal = 0;

    std::vector<HarmonicaSoundData> m_soundData;

    QString m_file;

public:
    QString file() const;
    void setFile(const QString& file);
    Q_SIGNAL void fileChanged(const QString& file);

    int bpm() const;
    void setBpm(int beatsPerMinute);
    Q_SIGNAL void bpmChanged();

    int bpmDelta() const;
    void setBpmDelta(int delta);
    Q_SIGNAL void bpmDeltaChanged();
    Q_SIGNAL void bpmTotalChanged();

    // a change in bpm will trigger a recalculate.
    void precalculate();

    // starts emmiting soundData.
    void start();
    void stop();
    Q_SIGNAL void soundData(const HarmonicaSoundData& soundData);
};
