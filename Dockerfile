FROM rust:1.81.0-slim-bullseye as builder

WORKDIR /usr/src/pandas-pouch

RUN apt-get update && \
    apt-get install -y \
    libssl-dev \
    pkg-config \
    protobuf-compiler && \
    rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY proto ./proto
COPY config ./config
COPY build.rs .

RUN cargo build --release

FROM debian:bullseye-slim

WORKDIR /usr/local/bin

RUN apt-get update && \
    apt-get install -y ca-certificates &&  \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/pandas-pouch/target/release/pandas-pouch /usr/local/bin/pandas-pouch
COPY --from=builder /usr/src/pandas-pouch/config /usr/src/pandas-pouch/config

CMD ["pandas-pouch"]