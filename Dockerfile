FROM --platform=amd64 rust:1.72-alpine as builder
WORKDIR /build

# RUN apt-get update && apt-get install -y build-essential musl-dev musl-tools gcc-x86-64-linux-gnu && apt-get clean && rm -rf /var/lib/apt/lists/*
RUN apk update && apk upgrade && apk add --no-cache musl-dev
RUN rustup target add x86_64-unknown-linux-musl

ENV CARGO_BUILD_RUSTFLAGS="-C target-feature=+crt-static"

COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && touch src/main.rs
RUN cargo fetch --target x86_64-unknown-linux-musl --locked

COPY . ./
RUN cargo build --target x86_64-unknown-linux-musl --release --locked

FROM gcr.io/distroless/static:latest
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/valfisk /valfisk

USER nonroot

CMD ["/valfisk"]
