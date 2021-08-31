#pragma once

#include "HarmonicaSoundData.h"

#include <QString>
#include <vector>

/* Parse the harmonica from a .tcg file */
class HarmonicaParser {
private:
    QString m_errorString;
    QString m_file;
    int m_bpm;
    std::vector<HarmonicaSoundData> m_data;

    bool parseBpm(const QString& line);
    bool parseWait(const QString& line);
    bool parseNote(const QString& line);


public:
    void setFile(const QString& file);
    bool parse();
    int bpm() const;
    std::vector<HarmonicaSoundData> data() const;

#ifdef TEST_BUILD
    static int testParsing();
#endif

};
