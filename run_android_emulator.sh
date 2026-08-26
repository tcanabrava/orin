export ANDROID_HOME="$HOME/Android/Sdk"
export JAVA_HOME=/opt/android-studio/jbr
export PATH="$ANDROID_HOME/emulator:$ANDROID_HOME/platform-tools:$PATH"

emulator -avd harmonicon-test &          # the AVD I created is still there
adb wait-for-device

cd packaging/android
./gradlew installRelease -Pharmonicon.abis=x86_64

adb shell am start -n \
    io.github.tcanabrava.harmonicon/com.google.androidgamesdk.GameActivity
