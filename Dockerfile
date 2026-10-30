# wow-ui-sim Dockerfile
#
# Build: docker build -t ghcr.io/osso/wow-ui-sim .
# Usage: docker run --rm -v ./MyAddon:/app/Interface/AddOns/MyAddon ghcr.io/osso/wow-ui-sim run-tests MyAddon

# =============================================================================
# Build Stage
# =============================================================================
ARG WOW_UI_SIM_PROFILE=retail
FROM rust:1.92-bookworm AS builder
ARG WOW_UI_SIM_PROFILE

# Install system build dependencies:
# - git: needed by some build scripts
# - pkg-config + cmake: required by C-backed Rust crates (mlua vendored Lua, wgpu)
RUN apt-get update && apt-get install -y --no-install-recommends \
    clang \
    mold \
    git \
    pkg-config \
    cmake \
    libicu-dev \
    libudev-dev \
    fonts-dejavu-core \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy dependency manifests first so Docker can cache the dep-fetch layer.
# iced-wgpu-patched/ is a [patch.crates-io] override and must be present
# before `cargo fetch` or any build step.
COPY Cargo.toml Cargo.lock ./
# Published binaries use the portable CI config, never local target-cpu=native.
COPY .cargo/config.ci.toml .cargo/config.toml
COPY iced-wgpu-patched/ iced-wgpu-patched/
COPY iced-dynamic/ iced-dynamic/
COPY xtask/ xtask/
# Cargo resolves all workspace/path dependencies even for binary-only builds.
# Keep every local crate in the cached fetch layer as the workspace grows.
COPY crates/ crates/

# Cargo validates explicitly declared targets before fetching dependencies.
# Derive empty sources from the manifest so adding targets cannot stale a list.
COPY scripts/docker-stub-cargo-targets.sh /tmp/docker-stub-cargo-targets.sh
RUN sh /tmp/docker-stub-cargo-targets.sh /build/Cargo.toml && cargo fetch --locked

# Copy the full source tree and build the simulator binary.
# Enable the retail profile and GUI rendering without sound.
COPY build.rs ./
# Retail selects retail-12-1-5, including the native ICU build/runtime dependency.
COPY build/ build/
COPY native/ native/
COPY data/ data/
# Overwrite every binary stub with real sources. Test/example/bench stubs are
# manifest-only inputs: the binary-only build below never compiles them.
COPY src/ src/
RUN mkdir -p /spell-bundle /cli-bundle \
    && case "$WOW_UI_SIM_PROFILE" in \
        retail) cargo build --release --bin wow-sim --no-default-features --features client-retail,gui --locked ;; \
        wowforever) cargo build --release --bin wow-sim --bin wow-cli --no-default-features --features client-wowforever,gui,tools --locked \
            && /build/target/release/wow-cli casc sync-forever-spells --source cdn --output /spell-bundle/spells.json \
            && cp /build/target/release/wow-cli /cli-bundle/wow-cli ;; \
        *) echo "Unsupported profile: $WOW_UI_SIM_PROFILE" >&2; exit 1 ;; \
    esac \
    && strip /build/target/release/wow-sim \
    && mkdir /icu-runtime \
    && cp -L /usr/lib/x86_64-linux-gnu/libicui18n.so.72 \
             /usr/lib/x86_64-linux-gnu/libicuuc.so.72 \
             /usr/lib/x86_64-linux-gnu/libicudata.so.72 /icu-runtime/

# =============================================================================
# BlizzardUI Stage — sparse-checkout from Gethe/wow-ui-source
# =============================================================================
FROM alpine/git AS blizzard-ui
ARG WOW_UI_SIM_PROFILE

# Never publish a cache whose tree differs from the selected profile manifest.
COPY data/blizzard-ui-files/retail.txt data/blizzard-ui-files/wowforever.txt /tmp/manifests/
RUN case "$WOW_UI_SIM_PROFILE" in \
        retail) revision=b9be9379719047977fca4dba1bb1d9988e8840b3; version=12.1.5.70448 ;; \
        wowforever) revision=ca8368f2f2380cbe696854f7f2646387c278a61b; version=1.60.1.70532 ;; \
        *) echo "Unsupported profile: $WOW_UI_SIM_PROFILE" >&2; exit 1 ;; \
    esac \
    && git init /wow-ui-source \
    && cd /wow-ui-source \
    && git remote add origin https://github.com/Gethe/wow-ui-source.git \
    && git fetch --depth=1 origin "$revision" \
    && git sparse-checkout init --cone \
    && git sparse-checkout set Interface/AddOns \
    && git checkout --detach FETCH_HEAD \
    && test "$(git rev-parse HEAD)" = "$revision" \
    && git ls-tree -r --name-only HEAD Interface/AddOns/ | cut -d/ -f3- | LC_ALL=C sort > /tmp/actual.txt \
    && LC_ALL=C sort "/tmp/manifests/$WOW_UI_SIM_PROFILE.txt" > /tmp/expected.txt \
    && diff -u /tmp/expected.txt /tmp/actual.txt \
    && while IFS= read -r entry; do test -f "Interface/AddOns/$entry" || exit 1; done < "/tmp/manifests/$WOW_UI_SIM_PROFILE.txt" \
    && manifest_sha256="$(sha256sum "/tmp/manifests/$WOW_UI_SIM_PROFILE.txt" | cut -d' ' -f1)" \
    && printf 'profile=%s\nversion=%s\nsource=gethe-image-build\ngethe_revision=%s\nmanifest_sha256=%s\nfallback=none\n' "$WOW_UI_SIM_PROFILE" "$version" "$revision" "$manifest_sha256" > Interface/AddOns/.wow-ui-sim-blizzard-ui-provenance \
    && touch Interface/AddOns/.wow-ui-sim-blizzard-ui-complete \
    && rm -rf /wow-ui-source/.git

# =============================================================================
# Runtime Stage
# =============================================================================
FROM debian:bookworm-slim
ARG WOW_UI_SIM_PROFILE

RUN apt-get update && apt-get install -y --no-install-recommends \
    libvulkan1 mesa-vulkan-drivers ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy stripped binary from build stage.
COPY --from=builder /build/target/release/wow-sim /app/wow-sim
COPY --from=builder /cli-bundle/ /app/

# Debian 12 ICU code and data libraries, from the same builder package set.
COPY --from=builder /icu-runtime/ /usr/lib/x86_64-linux-gnu/

# Copy data directories from the build context.
# These are read at runtime and are NOT compiled into the binary.
#
# BlizzardUI: Blizzard's base UI Lua/XML, placed in the cache path the
# simulator checks. The .wow-ui-sim-blizzard-ui-complete marker is
# created in the blizzard-ui stage and tells the runtime the cache
# is ready, skipping the CASC sync attempt.
COPY --from=blizzard-ui /wow-ui-source/Interface/AddOns/ /root/.cache/wow-ui-sim/blizzard-ui/${WOW_UI_SIM_PROFILE}/AddOns/
COPY --from=builder /spell-bundle/ /app/spells/wowforever/

# TestFramework: assertion library loaded automatically by `run-tests`
COPY Interface/AddOns/TestFramework/ /app/Interface/AddOns/TestFramework/

# DejaVu fonts for text shaping fallback when CASC is unavailable.
# fontdb::load_system_fonts() picks these up from the standard path.
COPY --from=builder /usr/share/fonts/truetype/dejavu/ /usr/share/fonts/truetype/dejavu/

# Skip SavedVariables loading — no WTF directory is available in the image.
ENV WOW_SIM_NO_SAVED_VARS=1 \
    WOW_SIM_SOFTWARE_RENDER=1

ENTRYPOINT ["/app/wow-sim"]
