FROM rustembedded/cross:i686-linux-android-0.2.1

WORKDIR /tmp

ENV HOST_PLATFORM linux-x86_64
ENV NDK_VERSION android-ndk-r22b
ENV ANDROID_NDK_ROOT "/tmp/${NDK_VERSION}"
ENV FILENAME "${NDK_VERSION}-${HOST_PLATFORM}.zip"

RUN apt-get update -y
RUN apt-get install -y wget unzip

RUN wget -q http://dl.google.com/android/repository/$FILENAME -O $FILENAME 
RUN unzip -q $FILENAME

RUN git clone https://github.com/zeromq/libzmq.git

RUN cd libzmq/builds/android && ./build.sh x86

ENV PKG_CONFIG_PATH=/tmp/libzmq/builds/android/prefix/x86/lib/pkgconfig