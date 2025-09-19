#!/bin/sh

export ANDROID_HOME=$HOME/Android/Sdk && \
    export NDK_VERSION=29.0.13846066 && \
    export ANDROID_NDK_ROOT=$HOME/Android/Sdk/ndk/${NDK_VERSION} && \
    export PATH=$PATH:$ANDROID_HOME/platform-tools && \
    export PATH=$PATH:$HOME/gradle-8.14.3/bin

#adb shell "mkdir -p /data/local/tmp/tools"
#adb push "$ANDROID_NDK_ROOT"/toolchains/llvm/prebuilt/linux-x86_64/lib/clang/21/lib/linux/x86_64/* /data/local/tmp/tools

#cargo install xbuild --git https://github.com/rust-mobile/xbuild.git --branch master
#rustup target add aarch64-linux-android
#rustup target add x86_64-linux-android
#x doctor
#x devices
#x lldb --device adb:emulator-5554 --arch x64 --debug
x run --device adb:emulator-5554 --arch x64 --debug
#x run --device adb:emulator-5554 --arch x64 --release
#x build --platform android --arch arm64 --store sideload
#x build --platform android --arch arm64 --store sideload --release