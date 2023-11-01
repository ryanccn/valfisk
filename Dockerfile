FROM rust:1.72-alpine as common-build
ARG TARGETPLATFORM
RUN \
  if [ "$TARGETPLATFORM" = "linux/amd64" ]; then target="x86_64-unknown-linux-musl"; \
  elif [ "$TARGETPLATFORM" = "linux/arm64" ]; then target="aarch64-unknown-linux-musl"; \
  else echo "Unsupported platform ${TARGETPLATFORM}!" && exit 1; \
  fi && \
  echo "$target" > /tmp/rust-target

RUN apk update && apk upgrade && apk add --no-cache musl-dev
RUN rustup target add "$(cat /tmp/rust-target)"
RUN cargo install cargo-chef --locked

WORKDIR /build

FROM common-build AS planner

COPY . ./
RUN cargo chef prepare --recipe-path recipe.json

FROM common-build AS builder
ENV CARGO_BUILD_RUSTFLAGS="-C target-feature=+crt-static"

COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --recipe-path recipe.json --release --locked --target "$(cat /tmp/rust-target)"

COPY . ./
RUN cargo build --release --locked --target "$(cat /tmp/rust-target)" --bin valfisk && \
  mv "./target/$(cat /tmp/rust-target)/release/valfisk" ./valfisk

FROM gcr.io/distroless/static:latest
COPY --from=builder /build/valfisk /valfisk

USER nonroot
CMD ["/valfisk"]
