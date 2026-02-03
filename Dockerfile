
FROM rust:1-slim-bookworm AS builder

# install bevy dependencies
RUN apt update && apt install -y --no-install-recommends \
    libasound2-dev \
    libudev-dev \
    libwayland-dev \
    libxkbcommon-dev \
    libx264-164 libx264-dev \
    pkg-config

WORKDIR /app

# Build application
COPY crates/server crates/server
COPY crates/shared crates/shared
COPY Cargo.toml Cargo.toml

RUN cargo build --release --bin server

COPY entrypoint.sh .

# We do not need the Rust toolchain to run the binary!
FROM rust:1-slim-bookworm AS runtime

RUN apt update && apt install -y --no-install-recommends \
    libasound2-dev \
    libudev-dev \
    libwayland-dev \
    libxkbcommon-dev \
    libx264-164 libx264-dev \
    pkg-config

WORKDIR /app
COPY --from=builder /app/target/release/server /usr/local/bin
COPY --from=builder /app/entrypoint.sh .
EXPOSE 4100
EXPOSE 5888/udp

ENTRYPOINT ["./entrypoint.sh"]
# ENTRYPOINT ["/usr/local/bin/server"]
