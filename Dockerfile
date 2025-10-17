FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig linux-headers

RUN cargo install sqlx-cli --no-default-features --features native-tls,postgres

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app

COPY . .

RUN cargo build -j4 --no-default-features --target x86_64-unknown-linux-musl --release

FROM alpine:latest

RUN apk add --no-cache openssl ca-certificates postgresql-client

RUN addgroup -S app && adduser -S app -G app

WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/app_example ./
COPY --from=builder /app/migrations ./migrations
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx

RUN chown -R app:app /app

USER app

CMD ["sh", "-c", "sqlx migrate run && ./app_example"]