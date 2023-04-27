FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app
RUN apt update && apt install lld clang -y


FROM chef as PLANNER
COPY . .
# Generate chef's "lock" file
RUN cargo chef prepare --recipe-path recipe.json


FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build deps.
RUN cargo chef cook --release --recipe-path recipe.json

# Build App
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --bin zero2prod

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
