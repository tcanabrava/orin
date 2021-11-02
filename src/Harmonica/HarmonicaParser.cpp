#include "HarmonicaParser.h"

#include <QFileInfo>
#include <QFile>
#include <QTextStream>
#include <QDebug>

void HarmonicaParser::setFile(const QString& file)
{
    m_file = file;
}

bool HarmonicaParser::parse()
{
    QFileInfo fInfo(m_file);
    if (!fInfo.exists()) {
        m_errorString = QObject::tr("There's no such file: %1").arg(m_file);
        return false;
    }

    QFile harmonicaFile(m_file);
    if (!harmonicaFile.open(QIODevice::ReadOnly)) {
        m_errorString = QObject::tr("Could not open file, check permissions");
        return false;
    }

    int lineNr = 0;

    QTextStream streamReader(&harmonicaFile);
    while (!streamReader.atEnd()) {
        lineNr += 1;

        const QString line = streamReader.readLine().trimmed();
        if (line.startsWith(QLatin1Char('#'))) {
            continue;
        }
        if (line.isEmpty()) {
            continue;
        }

        auto methodPtr = line.startsWith("beats_per_minute") ? &HarmonicaParser::parseBpm
                    : line.startsWith("lyrics") ? &HarmonicaParser::parseLyrics
                    : line.startsWith("about") ? &HarmonicaParser::parseAbout
                    : line.startsWith("wait") ? &HarmonicaParser::parseWait
                    : &HarmonicaParser::parseNote;

        const bool result = (this->*methodPtr)(line, lineNr);
        if (!result) {
            return false;
        }
    }
    return true;
}

bool HarmonicaParser::parseAbout(const QString& line, int lineNr)
{
    QStringList bpmLine = line.split('=');
    if (bpmLine.count() != 2) {
        m_errorString = QObject::tr("File malformed. about = number, line = %1")
            .arg(lineNr);
        return false;
    }

    m_aboutUrl = bpmLine[1].trimmed();
    return true;
}

bool HarmonicaParser::parseLyrics(const QString& line, int lineNr)
{
    QStringList bpmLine = line.split('=');
    if (bpmLine.count() != 2) {
        m_errorString = QObject::tr("File malformed. lyrics = number, line = %1")
            .arg(lineNr);
        return false;
    }

    m_lyricsUrl = bpmLine[1].trimmed();
    return true;
}

bool HarmonicaParser::parseBpm(const QString& line, int lineNr)
{
    QStringList bpmLine = line.split('=');
    if (bpmLine.count() != 2) {
        m_errorString = QObject::tr("File malformed. beats_per_minute = number, line = %1")
            .arg(lineNr);
        return false;
    }

    bool conversionOk = false;
    const int value = bpmLine[1].toInt(&conversionOk);
    if (!conversionOk) {
        m_errorString = QObject::tr("File malformed. beats_per_minute = number, line = %1")
            .arg(lineNr);
        return false;
    }

    m_bpm = value;
    return true;
}

bool HarmonicaParser::parseWait(const QString& line, int lineNr)
{
    QStringList waitLine = line.split(' ');

    if (waitLine.count() != 2) {
        m_errorString = QObject::tr("File malformed. wait [number of beats], line = %1")
            .arg(lineNr);

        return false;
    }

    std::optional<HarmonicaSoundData::Duration> duration = HarmonicaSoundData::durationFromString(waitLine[1]);
    if (!duration.has_value()) {
        m_errorString = QObject::tr("Note duration can only accept 1, 1/2, 1/4 or 1/8 beats."
        " \n for everything else, use ligatures, line = %1")
            .arg(lineNr);

        return false;
    }

    HarmonicaSoundData soundData;
    soundData.soundType = HarmonicaSoundData::NONE;
    soundData.duration = duration.value();
    m_data.push_back(soundData);
    return true;
}

bool HarmonicaParser::parseNote(const QString& line, int lineNr)
{
    QStringList lineSplit = line.split(' ', QString::SkipEmptyParts);
    if (lineSplit.count() < 2 || lineSplit.count() > 5) {
        m_errorString = QObject::tr("Number of elements on the note is incorrect, line = %1")
            .arg(lineNr);
        return false;
    }

    // first part
    QStringList holes = lineSplit[0].split(';', QString::SkipEmptyParts);
    QList<int> holesResult;
    for (const QString& stringHole : holes) {
        bool conversionOk = false;
        const int value = stringHole.toInt(&conversionOk);
        if (!conversionOk) {
            m_errorString = QObject::tr("File malformed. hole is not numeric. line = %1")
                .arg(lineNr);
            return false;
        }
        if (value < 1 || value > 10) {
            m_errorString = QObject::tr("File malformed. hole is out of range. line = %1")
                .arg(lineNr);
            return false;
        }

        holesResult.push_back(value);
    }

    std::optional<HarmonicaSoundData::Duration> duration = HarmonicaSoundData::durationFromString(lineSplit.last());
    if (!duration.has_value()) {
        m_errorString = QObject::tr("Note duration can only accept 1, 1/2, 1/4 or 1/8 beats. \n for everything else, use ligatures, line = %1")
            .arg(lineNr);
        return false;
    }

    const bool hasLigature = lineSplit.contains("ligature");
    const bool isBlow = lineSplit.contains("blow");
    const bool isDraw = lineSplit.contains("draw");

    if (isBlow && isDraw) {
        m_errorString = QObject::tr("A line can't be both blow and draw, line = %1")
            .arg(lineNr);
        return false;
    }

    HarmonicaSoundData data;
    data.duration = duration.value();
    data.holes = holesResult;
    data.soundType = isBlow
        ? HarmonicaSoundData::SoundType::BLOW
        : HarmonicaSoundData::SoundType::DRAW;
    data.ligature = hasLigature;

    m_data.push_back(data);

    return true;
}

int HarmonicaParser::bpm() const
{
    return m_bpm;
}

std::vector<HarmonicaSoundData> HarmonicaParser::data() const
{
    return m_data;
}

QString HarmonicaParser::errorString() const
{
    return m_errorString;
}

QString HarmonicaParser::lyricsUrl() const
{
    return m_lyricsUrl;
}

QString HarmonicaParser::aboutUrl() const
{
    return m_aboutUrl;
}

#ifdef TEST_BUILD
#include <iostream>
int HarmonicaParser::testParsing() {
    HarmonicaParser parser;
    // Expected pass.
    if (!parser.parseBpm("beats_per_minute = 60", 0)) {
        std::cerr << "Bpm 60 not parsing";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
     if (!parser.parseBpm("beats_per_minute = 85", 0)) {
        std::cerr << "Bpm 85 not parsing";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
     if (!parser.parseBpm("beats_per_minute = 120", 0)) {
        std::cerr << "Bpm 120 not parsing";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }

    // Expect failures.
    if (parser.parseBpm("beats_per_minute = 1/16", 0)) {
        std::cerr << "Bpm 1/16 parsing";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
    if (parser.parseBpm("beats_per_minute = ABC", 0)) {
        std::cerr << "Bpm ABC parsing";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }

    // Expect pass
    if (!parser.parseWait("wait 1", 0)) {
        std::cerr << "Wait 1 not parsed";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
    if (!parser.parseWait("wait 1/2", 0)) {
        std::cerr << "Wait 1/2 not parsed";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
    if (!parser.parseWait("wait 1/4", 0)) {
        std::cerr << "Wait 1/4 not parsed";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
    if (!parser.parseWait("wait 1/8", 0)) {
        std::cerr << "Wait 1/8 not parsed";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }

    // expected failures
    if (parser.parseWait("1 wait", 0)) {
        std::cerr << "1 wait parsed";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
    if (parser.parseWait("wait 1/18", 0)) {
        std::cerr << "Wait 1/18 parsed";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
    if (parser.parseWait("wait 1/3", 0)) {
        std::cerr << "Wait 1/3 parsed";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
    if (parser.parseWait("wait", 0)) {
        std::cerr << "Wait parsed";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }

    // expected pass
    if (!parser.parseNote("1 blow 1/2", 0)) {
        std::cerr << "Parse note failed 1";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
    if (!parser.parseNote("1 draw 1/2", 0)) {
        std::cerr << "Parse note failed 2";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
    if (!parser.parseNote("1 ligature blow 1/2", 0)) {
        std::cerr << "Parse note failed 3";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
    if (!parser.parseNote("1 ligature draw 1/2", 0)) {
        std::cerr << "Parse note failed 4";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
    if (!parser.parseNote("1;2;3 draw 1/2", 0)) {
        std::cerr << "Parse note failed 5";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
    if (!parser.parseNote("4;5;6 ligature draw 1/2", 0)) {
        std::cerr << "Parse note failed 6";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
    return 0;
}

#endif
