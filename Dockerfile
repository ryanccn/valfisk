FROM --platform=amd64 rust:1.72-alpine as common-build

RUN apk update && apk upgrade && apk add --no-cache musl-dev
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo install cargo-chef --locked

WORKDIR /build

FROM common-build AS planner

COPY . ./
RUN cargo chef prepare --recipe-path recipe.json

FROM common-build AS builder
ENV CARGO_BUILD_RUSTFLAGS="-C target-feature=+crt-static"

COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --recipe-path recipe.json --release --locked --target x86_64-unknown-linux-musl

COPY . ./
RUN cargo build --release --locked --target x86_64-unknown-linux-musl --bin valfisk

FROM gcr.io/distroless/static:latest
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/valfisk /valfisk

USER nonroot

CMD ["/valfisk"]
