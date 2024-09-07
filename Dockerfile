# Use the official Rust image from Docker Hub
FROM rust:latest AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/

RUN cargo build --release

# Use a minimal base image for the final stage
FROM debian:latest
WORKDIR /app
COPY --from=builder /app/target/release/backup .

RUN apt-get update && apt-get install -y openssh-client libssh-dev && rm -rf /var/lib/apt/lists/*

# Run the binary
CMD ["./backup"]
