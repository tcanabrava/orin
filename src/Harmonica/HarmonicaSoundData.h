#pragma once


#include <vector>
#include <chrono>
#include <optional>

#include <QList>
#include <QDebug>
#include <QObject>

/* of course it's possible to use
 * a better representation of music,
 * but for a harmonica, it's simpler to specify
 * like this.
 */
struct HarmonicaSoundData {
    Q_GADGET
    Q_PROPERTY(QList<int> holes MEMBER holes)

public:
    enum SoundType {
        NONE,
        DRAW,
        BLOW
    };
    Q_ENUM(SoundType);

    enum Duration {
        BEAT_1,     // 1   beat
        BEAT_1_2,   // 1/2 beat
        BEAT_1_4,   // 1/4 beat
        BEAT_1_8,   // 1/8 beat
    };
    Q_ENUM(Duration);

    static std::optional<HarmonicaSoundData::Duration> durationFromString(const QString& str);

    QList<int> holes;

    // The duration in beats of the note.
    Duration duration = Duration::BEAT_1;

    // This is filled by the HarmonicaMusic, that knows
    // how many BPM the song has. as soon as the music is loaded
    // the BPM is calculated and we fill this.
    std::chrono::milliseconds mappedDuration;

    // Is this a pause, a blow, or a draw?
    SoundType soundType = SoundType::NONE;

    // Is this a ligature?
    bool ligature = false;
};

// Operator for debugging.
QDebug& operator<<(QDebug& dbg, const HarmonicaSoundData &data);
