FROM rust:1.69 AS builder

WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

# TODO: https://github.com/emk/rust-musl-builder
FROM debian:bullseye-slim AS runtime
WORKDIR /app

# Installing runtime dependencies and cleaning up the mess:
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/zero2prod zero2prod
COPY config config
ENV APP_ENVIRONMENT production

ENTRYPOINT ["./zero2prod"]
