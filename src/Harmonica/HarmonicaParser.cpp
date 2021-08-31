#include "HarmonicaParser.h"

#include <QFileInfo>
#include <QFile>
#include <QTextStream>

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

    QTextStream streamReader(&harmonicaFile);
    while (!streamReader.atEnd()) {
        const QString line = streamReader.readLine().trimmed().toLower();
        if (line.startsWith(QLatin1Char('#'))) {
            continue;
        }
        if (line.isEmpty()) {
            continue;
        }

        if (line.startsWith("beats_per_minute")) {
            if (parseBpm(line)) {
                continue;
            } else {
                return false;
            }
        }

        if (line.startsWith("wait")) {
            if (parseWait(line)) {
                continue;
            } else {
                return false;
            }
        }

        if (parseNote(line)) {
            continue;
        } else {
            return false;
        }
    }
    return true;
}

bool HarmonicaParser::parseBpm(const QString& line)
{
    QStringList bpmLine = line.split('=');
    if (bpmLine.count() != 2) {
        m_errorString = QObject::tr("File malformed. beats_per_minute = number");
        return false;
    }

    bool conversionOk = false;
    const int value = bpmLine[1].toInt(&conversionOk);
    if (!conversionOk) {
        m_errorString = QObject::tr("File malformed. beats_per_minute = number");
        return false;
    }

    m_bpm = value;
    return true;
}

bool HarmonicaParser::parseWait(const QString& line)
{
    QStringList waitLine = line.split(' ');

    if (waitLine.count() != 2) {
        m_errorString = QObject::tr("File malformed. wait [number of beats]");
        return false;
    }

    std::optional<HarmonicaSoundData::Duration> duration = HarmonicaSoundData::durationFromString(waitLine[1]);
    if (!duration.has_value()) {
        m_errorString = QObject::tr("Note duration can only accept 1, 1/2, 1/4 or 1/8 beats. \n for everything else, use ligatures");
        return false;
    }

    HarmonicaSoundData soundData;
    soundData.soundType = HarmonicaSoundData::NONE;
    soundData.duration = duration.value();
    m_data.push_back(soundData);
    return true;
}

bool HarmonicaParser::parseNote(const QString& line)
{
    QStringList lineSplit = line.split(' ', Qt::SkipEmptyParts);
    if (lineSplit.count() < 2 || lineSplit.count() > 5) {
        m_errorString = QObject::tr("Number of elements on the note is incorrect");
        return false;
    }

    // first part
    QStringList holes = lineSplit[0].split(';', Qt::SkipEmptyParts);
    std::vector<int> holesResult;
    for (const QString& stringHole : holes) {
        bool conversionOk = false;
        const int value = stringHole.toInt(&conversionOk);
        if (!conversionOk) {
            m_errorString = QObject::tr("File malformed. hole is not numeric.");
            return false;
        }
        if (value < 1 || value > 10) {
            m_errorString = QObject::tr("File malformed. hole is out of range.");
            return false;
        }

        holesResult.push_back(value);
    }

    std::optional<HarmonicaSoundData::Duration> duration = HarmonicaSoundData::durationFromString(lineSplit.last());
    if (!duration.has_value()) {
        m_errorString = QObject::tr("Note duration can only accept 1, 1/2, 1/4 or 1/8 beats. \n for everything else, use ligatures");
        return false;
    }

    const bool hasLigature = lineSplit.contains("ligature");
    const bool isBlow = lineSplit.contains("blow");
    const bool isDraw = lineSplit.contains("draw");

    if (isBlow && isDraw) {
        m_errorString = QObject::tr("A line can't be both blow and draw");
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

#ifdef TEST_BUILD
#include <iostream>
int HarmonicaParser::testParsing() {
    HarmonicaParser parser;
    // Expected pass.
    if (!parser.parseBpm("beats_per_minute = 60")) {
        std::cerr << "Bpm 60 not parsing";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
     if (!parser.parseBpm("beats_per_minute = 85")) {
        std::cerr << "Bpm 85 not parsing";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
     if (!parser.parseBpm("beats_per_minute = 120")) {
        std::cerr << "Bpm 120 not parsing";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }

    // Expect failures.
    if (parser.parseBpm("beats_per_minute = 1/16")) {
        std::cerr << "Bpm 1/16 parsing";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
    if (parser.parseBpm("beats_per_minute = ABC")) {
        std::cerr << "Bpm ABC parsing";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }

    // Expect pass
    if (!parser.parseWait("wait 1")) {
        std::cerr << "Wait 1 not parsed";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
    if (!parser.parseWait("wait 1/2")) {
        std::cerr << "Wait 1/2 not parsed";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
    if (!parser.parseWait("wait 1/4")) {
        std::cerr << "Wait 1/4 not parsed";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
    if (!parser.parseWait("wait 1/8")) {
        std::cerr << "Wait 1/8 not parsed";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }

    // expected failures
    if (parser.parseWait("1 wait")) {
        std::cerr << "1 wait parsed";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
    if (parser.parseWait("wait 1/18")) {
        std::cerr << "Wait 1/18 parsed";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
    if (parser.parseWait("wait 1/3")) {
        std::cerr << "Wait 1/3 parsed";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
    if (parser.parseWait("wait")) {
        std::cerr << "Wait parsed";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }

    // expected pass
    if (!parser.parseNote("1 blow 1/2")) {
        std::cerr << "Parse note failed 1";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
    if (!parser.parseNote("1 draw 1/2")) {
        std::cerr << "Parse note failed 2";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
    if (!parser.parseNote("1 ligature blow 1/2")) {
        std::cerr << "Parse note failed 3";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
    if (!parser.parseNote("1 ligature draw 1/2")) {
        std::cerr << "Parse note failed 4";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
    if (!parser.parseNote("1;2;3 draw 1/2")) {
        std::cerr << "Parse note failed 5";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
    if (!parser.parseNote("4;5;6 ligature draw 1/2")) {
        std::cerr << "Parse note failed 6";
        std::cerr << parser.m_errorString.toStdString();
        return 1;
    }
    return 0;
}

#endif
