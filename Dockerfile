# syntax=docker/dockerfile:1.7

ARG FFMPEG_VERSION=7.1

FROM alpine:3.23 AS ffmpeg-alpine
ARG FFMPEG_VERSION
RUN apk add --no-cache \
    bash \
        build-base \
        ca-certificates \
        curl \
        git \
        nasm \
        pkgconf \
        xz \
        yasm \
        zlib-dev
RUN git clone --depth 1 https://code.videolan.org/videolan/x264.git /tmp/x264 \
    && cd /tmp/x264 \
    && bash ./configure --prefix=/opt/ffmpeg --enable-static --disable-shared --enable-pic --disable-cli \
    && make -j"$(nproc)" \
    && make install
RUN curl -fsSL "https://ffmpeg.org/releases/ffmpeg-${FFMPEG_VERSION}.tar.xz" | tar -xJ -C /tmp \
    && cd "/tmp/ffmpeg-${FFMPEG_VERSION}" \
    && PKG_CONFIG_PATH=/opt/ffmpeg/lib/pkgconfig ./configure \
        --prefix=/opt/ffmpeg \
        --enable-gpl \
        --enable-libx264 \
        --disable-autodetect \
        --disable-debug \
        --disable-doc \
        --disable-network \
        --disable-programs \
        --enable-static \
        --disable-shared \
        --disable-demuxers \
        --enable-demuxer=image_jpeg_pipe,image_png_pipe,image_webp_pipe,image_bmp_pipe,image_gif_pipe,mov,matroska \
        --disable-muxers \
        --enable-muxer=mov,mp4,matroska,flv \
    && make -j"$(nproc)" \
    && make install

FROM rust:1-alpine3.23 AS builder-none
WORKDIR /app
RUN apk add --no-cache build-base pkgconf
COPY Cargo.toml Cargo.lock build.rs ./
COPY src ./src
COPY sass ./sass
COPY static ./static
RUN cargo build --release --locked

FROM rust:1-alpine3.23 AS builder-ugoira
WORKDIR /app
RUN apk add --no-cache build-base pkgconf
COPY --from=ffmpeg-alpine /opt/ffmpeg /opt/ffmpeg
ENV FFMPEG_PREFIX=/opt/ffmpeg
ENV PKG_CONFIG_PATH=/opt/ffmpeg/lib/pkgconfig
ENV FFMPEG_STATIC=1
COPY Cargo.toml Cargo.lock build.rs ./
COPY src ./src
COPY sass ./sass
COPY static ./static
COPY ugoira.cpp ./ugoira.cpp
RUN cargo build --release --locked --features ugoira

FROM alpine:3.23 AS alpine
RUN apk add --no-cache ca-certificates
RUN addgroup -S reapixa \
    && adduser -S -G reapixa -h /home/reapixa reapixa \
    && mkdir -p /home/reapixa \
    && chown -R reapixa:reapixa /home/reapixa
WORKDIR /home/reapixa
COPY --from=builder-none /app/target/release/reapixa /usr/local/bin/reapixa
ENV PORT=8000
ENV HOST=
ENV COOKIE=
EXPOSE 8000
USER reapixa
ENTRYPOINT ["/bin/sh", "-ec", "if [ -n \"$COOKIE\" ]; then exec reapixa --port \"$PORT\" --host \"${HOST:-http://localhost:$PORT}\" --cookie \"$COOKIE\"; else exec reapixa --port \"$PORT\" --host \"${HOST:-http://localhost:$PORT}\"; fi"]

FROM alpine:3.23 AS alpine-ugoira
RUN apk add --no-cache ca-certificates
RUN addgroup -S reapixa \
    && adduser -S -G reapixa -h /home/reapixa reapixa \
    && mkdir -p /home/reapixa \
    && chown -R reapixa:reapixa /home/reapixa
WORKDIR /home/reapixa
COPY --from=builder-ugoira /app/target/release/reapixa /usr/local/bin/reapixa
ENV PORT=8000
ENV HOST=
ENV COOKIE=
EXPOSE 8000
USER reapixa
ENTRYPOINT ["/bin/sh", "-ec", "if [ -n \"$COOKIE\" ]; then exec reapixa --port \"$PORT\" --host \"${HOST:-http://localhost:$PORT}\" --cookie \"$COOKIE\"; else exec reapixa --port \"$PORT\" --host \"${HOST:-http://localhost:$PORT}\"; fi"]

FROM alpine:3.23 AS final
