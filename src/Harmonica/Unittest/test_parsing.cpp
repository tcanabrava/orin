#include "../HarmonicaParser.h"

int main() {
    int ret = HarmonicaParser::testParsing();
    if (ret != 0) {
        return ret;
    }
}
