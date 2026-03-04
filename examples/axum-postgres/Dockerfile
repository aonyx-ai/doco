# Build stage
FROM rust:1.85.0-slim AS builder

WORKDIR /app

# Copy the Cargo.toml and Cargo.lock
COPY Cargo.toml Cargo.lock ./

# Cache the workspace
# Copy the source code
COPY examples/axum-postgres examples/axum-postgres
COPY crates crates

# Build the example
RUN cd examples/axum-postgres && cargo build --release

# Runtime stage
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Copy server binary
COPY --from=builder /app/examples/axum-postgres/target/release/axum-postgres server

# Configure a health check for the server
HEALTHCHECK CMD curl --fail http://localhost:3000 || exit 1

# Configure the axum server
ENV RUST_BACKTRACE=full
EXPOSE 3000

# Start the server
CMD ["/app/server"]
