# ===========================
# 1. Builder Stage
# ===========================
FROM rust:latest AS builder

WORKDIR /app

# Install required build tools
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Pre-copy Cargo files for caching dependencies
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build release binary
RUN cargo build --release


# ===========================
# 2. Runtime Stage
# ===========================
FROM debian:stable-slim

WORKDIR /app

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y libssl-dev && apt-get clean

# Copy binary from builder
COPY --from=builder /app/target/release/api_tst /app/api_tst

# Copy SQLite database
COPY movies.db /app/movies.db

# Expose port
EXPOSE 8080

# Run server
CMD ["./api_tst"]
