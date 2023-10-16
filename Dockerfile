FROM rust:1.73.0 AS builder
WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . /app
ENV SQLX_OFFLINE true
RUN cargo build --release


FROM rust:1.73.0-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/zero-to-production zero-to-production
COPY configuration configuration
ENV APP_ENVIRONMENT production

EXPOSE 8000
ENTRYPOINT ["./zero-to-production"]