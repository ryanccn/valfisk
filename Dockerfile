FROM rust:1.72-slim as builder
ARG TARGETPLATFORM
WORKDIR /build

RUN set -ex; \
    case "$TARGETPLATFORM" in \
        "linux/amd64") target='x86_64-unknown-linux-musl' ;; \
        "linux/arm64") target='aarch64-unknown-linux-musl' ;; \
        *) echo >&2 "error: unsupported $TARGETPLATFORM architecture"; exit 1 ;; \
    esac; \
    echo "$target" > /tmp/target-spec

RUN apt-get update && apt-get install -y build-essential musl-dev musl-tools && apt-get clean && rm -rf /var/lib/apt/lists/*
# RUN apk update && apk upgrade && apk add --no-cache musl-dev
RUN rustup target add "$(cat /tmp/target-spec)"

ENV CARGO_BUILD_RUSTFLAGS="-C target-feature=+crt-static"

COPY . .
RUN cargo fetch --target "$(cat /tmp/target-spec)" --locked
RUN cargo build --target "$(cat /tmp/target-spec)" --release --locked
RUN cp "/build/target/$(cat /tmp/target-spec)/release/valfisk" /build/valfisk

FROM gcr.io/distroless/static:latest

COPY --from=builder /build/valfisk /valfisk
CMD ["/valfisk"]
