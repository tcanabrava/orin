#include "HarmonicaSoundData.h"

std::optional<HarmonicaSoundData::Duration>
HarmonicaSoundData::durationFromString(const QString& str)
{
    if (str == "1") {
        return HarmonicaSoundData::BEAT_1;
    } else if (str == "1/2") {
        return  HarmonicaSoundData::BEAT_1_2;
    } else if (str == "1/4") {
        return HarmonicaSoundData::BEAT_1_4;
    } else if (str == "1/8") {
        return HarmonicaSoundData::BEAT_1_8;
    }
    return {};
}

QDebug& operator<<(QDebug& dbg, const HarmonicaSoundData &data) {
    dbg << "Holes:[";
    bool padding = false;
    for (int hole : data.holes) {
        if (padding) {
            dbg << ",";
        }
        dbg << hole;
        padding = true;
    }
    dbg << "]";
    dbg << " " << data.duration << data.soundType;
    return dbg;
}
