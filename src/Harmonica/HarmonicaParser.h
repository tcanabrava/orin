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

    bool parseBpm(const QString& line, int lineNr);
    bool parseWait(const QString& line, int lineNr);
    bool parseNote(const QString& line, int lineNr);


public:
    void setFile(const QString& file);
    bool parse();
    int bpm() const;
    QString errorString() const;
    std::vector<HarmonicaSoundData> data() const;

#ifdef TEST_BUILD
    static int testParsing();
#endif

};
