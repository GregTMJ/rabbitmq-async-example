FROM rust:alpine AS chef

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig linux-headers

RUN cargo install sqlx-cli --no-default-features --features native-tls,postgres
RUN cargo install cargo-chef

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app

FROM chef AS planner

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

COPY . .
RUN cargo build -j4 --target x86_64-unknown-linux-musl --release --bin app_example

FROM alpine:latest

RUN apk add --no-cache openssl ca-certificates postgresql-client

RUN addgroup -S app && adduser -S app -G app

WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/app_example ./
COPY --from=builder /app/migrations ./migrations
COPY --from=chef /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx

RUN chown -R app:app /app

USER app

CMD ["sh", "-c", "sqlx migrate run && ./app_example"]