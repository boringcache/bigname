# syntax=docker/dockerfile:1.7

ARG RUST_VERSION=1.93.1

FROM rust:${RUST_VERSION}-bookworm AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends clang libclang-dev \
    && rm -rf /var/lib/apt/lists/*

ARG TARGETARCH
RUN set -eu; \
    case "$TARGETARCH" in \
      amd64) platform=x86_64-unknown-linux-musl; checksum=67c4a96dd237c1f518f6b36083f270f9976d516f1e57fce891755ea782e50006 ;; \
      arm64) platform=aarch64-unknown-linux-musl; checksum=821a86343191aa1cbab74bd42f9e93c9a63bf85e4742945f40d3ae84193c1c77 ;; \
      *) echo "Unsupported sccache architecture: $TARGETARCH" >&2; exit 1 ;; \
    esac; \
    curl --fail --location --silent --show-error --retry 3 \
      "https://github.com/mozilla/sccache/releases/download/v0.17.0/sccache-v0.17.0-$platform.tar.gz" \
      -o /tmp/sccache.tar.gz; \
    echo "$checksum  /tmp/sccache.tar.gz" | sha256sum -c -; \
    tar -xzf /tmp/sccache.tar.gz -C /tmp; \
    install -m 0755 "/tmp/sccache-v0.17.0-$platform/sccache" /usr/local/bin/sccache; \
    rm -rf /tmp/sccache.tar.gz "/tmp/sccache-v0.17.0-$platform"

WORKDIR /app

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY .cargo .cargo
COPY apps apps
COPY crates crates
COPY tools tools
COPY migrations migrations
COPY manifests manifests
COPY schema-v2 schema-v2

ARG BIGNAME_BUILD_SHA=unknown
ENV BIGNAME_BUILD_SHA=${BIGNAME_BUILD_SHA}

# Keep dependency sources and release objects together across source revisions.
# Copy the finished executables out of the mount into the image layer.
RUN --mount=type=cache,target=/build-cache,sharing=locked \
    CARGO_HOME=/build-cache/cargo cargo build --locked --release --workspace --bins \
        --target-dir /build-cache/target \
    && mkdir -p /out \
    && cp /build-cache/target/release/bigname-api /build-cache/target/release/phase-runner /out/ \
    && sccache --show-stats

FROM ubuntu:24.04 AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl libgcc-s1 libstdc++6 tini \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system --gid 10001 bigname \
    && useradd --system --uid 10001 --gid bigname --home-dir /app --create-home bigname

WORKDIR /app

COPY --from=builder /out/bigname-api /usr/local/bin/bigname-api
COPY --from=builder /out/phase-runner /usr/local/bin/phase-runner
COPY --from=builder --chown=bigname:bigname /app/manifests /app/manifests
COPY --chmod=0755 docker/entrypoint.sh /usr/local/bin/bigname

ENV BIGNAME_API_BIND_ADDR=0.0.0.0:3000 \
    BIGNAME_PHASE_RUNNER_MANIFESTS_ROOT=/app/manifests/mainnet \
    RUST_LOG=info

EXPOSE 3000

USER bigname

ENTRYPOINT ["tini", "--", "bigname"]
CMD ["api"]
