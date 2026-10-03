# syntax=docker/dockerfile:1
FROM rust:1.98.1-bookworm AS build
# Default client-retail enables native-duration-formatting: ICU4C >=72.
# ALSA serves sound; OpenSSL/CMake/clang serve CASC/native dependencies.
RUN apt-get update && apt-get install -y --no-install-recommends \
    clang libclang-dev mold cmake pkg-config python3 patchelf \
    libasound2-dev libssl-dev libicu-dev && rm -rf /var/lib/apt/lists/*
ARG BUILD_ROOT
ARG TARGET_CACHE
ARG JOBS=8
ARG BIN=wow-sim
ARG RELEASE=0
ARG NO_DEFAULT_FEATURES=0
ARG FEATURES=""
ENV BUILD_ROOT=${BUILD_ROOT} BIN=${BIN} RELEASE=${RELEASE} \
    NO_DEFAULT_FEATURES=${NO_DEFAULT_FEATURES} FEATURES=${FEATURES} \
    CARGO_TARGET_DIR=${BUILD_ROOT}/target \
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=clang \
    RUSTFLAGS="-C target-cpu=x86-64 -C link-arg=-fuse-ld=mold -C link-arg=-Wl,--thread-count=4"
# Preserve env!(CARGO_MANIFEST_DIR): build at the originating checkout path.
WORKDIR ${BUILD_ROOT}
COPY ["source/", "${BUILD_ROOT}/"]
COPY builder.py /builder.py
RUN --mount=type=cache,id=wow-ui-sim-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=wow-ui-sim-git,target=/usr/local/cargo/git \
    --mount=type=cache,id=${TARGET_CACHE},target=${BUILD_ROOT}/target,sharing=locked \
    python3 /builder.py _container-build
FROM scratch AS artifact
COPY --from=build /out/ /
