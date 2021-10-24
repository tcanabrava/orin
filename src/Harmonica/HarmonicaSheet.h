#pragma once

#include <QObject>
#include <QString>
#include <QTimer>

#include <vector>

#include "HarmonicaSoundData.h"

/* Defines a harmonica song, completely. */
class HarmonicaSheet: public QObject {
    Q_OBJECT

    // Read write access from the interface
    Q_PROPERTY(QString file READ file WRITE setFile NOTIFY fileChanged)
    Q_PROPERTY(int bpm READ bpm WRITE setBpm NOTIFY bpmChanged)
    Q_PROPERTY(int bpmDelta READ bpmDelta WRITE setBpmDelta NOTIFY bpmDeltaChanged)
    Q_PROPERTY(bool running READ running WRITE setRunning NOTIFY runningChanged)

    // Read only access from the interface
    Q_PROPERTY(int bpmTotal READ bpmTotal NOTIFY bpmTotalChanged)
    Q_PROPERTY(bool ready READ ready NOTIFY readyChanged)
    Q_PROPERTY(int totalBeats READ totalBeats NOTIFY totalBeatsChanged)
    Q_PROPERTY(int currentBeat READ currentBeat NOTIFY currentBeatChanged)

private:
    // the speed of the music
    int m_bpm = 0;

    // change the delta if you want something faster or slower.
    int m_bpmDelta = 0;

    // the sum of m_bpm and m_bpmDelta
    int m_bpmTotal = 0;

    // the total number of beats of the song.
    int m_totalBeats = 0;

    // the position we are on the music right now.
    int m_currentBeat = 0;

    std::vector<HarmonicaSoundData> m_soundData;

    QString m_file;

    bool m_ready = false;
    bool m_running = false;

    // each timer tick, currIdx advances, till the music finishes.
    QTimer m_bpmTimer;
    int m_currIdx = 0;

    // starts emmiting soundData.
    void start();
    void stop();

    // triggered by the timer.
    void timerTick();

public:
    HarmonicaSheet();

    QString file() const;
    void setFile(const QString& file);
    Q_SIGNAL void fileChanged(const QString& file);

    int bpm() const;
    void setBpm(int beatsPerMinute);
    Q_SIGNAL void bpmChanged();

    int totalBeats() const;
    Q_SIGNAL void totalBeatsChanged();

    int currentBeat() const;
    Q_SIGNAL void currentBeatChanged();

    int bpmDelta() const;
    void setBpmDelta(int delta);

    int bpmTotal() const;
    Q_SIGNAL void bpmDeltaChanged();
    Q_SIGNAL void bpmTotalChanged();

    Q_SIGNAL void errorMessage(const QString& message);

    bool ready() const;
    void setReady(bool ready); /* not exported to Qml */
    Q_SIGNAL void readyChanged(bool ready);

    Q_SIGNAL void sendBeat(const HarmonicaSoundData& beat, int idx);

    // Starts or stops the song.
    bool running() const;
    void setRunning(bool running);
    Q_SIGNAL void runningChanged(bool running);

    void setSoundData(const std::vector<HarmonicaSoundData>& soundData);

    // a change in bpm will trigger a recalculate.
    void precalculate();
};

