add_library(OrinHarmonica STATIC
    HarmonicaSoundData.cpp
    HarmonicaParser.cpp
    HarmonicaSheet.cpp
)

target_link_libraries(
    OrinHarmonica
    PUBLIC Qt${QT_VERSION_MAJOR}::Core
)

if (BUILD_TESTS)
    target_compile_definitions(OrinHarmonica PUBLIC TEST_BUILD)
    add_subdirectory(Unittest)
endif()

