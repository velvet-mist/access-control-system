FROM rust:1.93 AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libudev-dev \
    python3 \
    python3-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY . .
RUN cargo build --release

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libudev1 \
    python3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/access-control-system /usr/local/bin/access-control-system

ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080
ENV PLC_TYPE=keyence
ENV RUN_EMBEDDED_PYTHON=false

EXPOSE 8080

CMD ["access-control-system"]
