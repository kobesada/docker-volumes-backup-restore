# Use the official Rust image from Docker Hub
FROM rust:latest AS builder

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release || true
RUN rm -rf src/*

# Copy the source code last, so that it gets rebuilt only if there are code changes
COPY src/ src/

# Build the Rust program
RUN cargo build --release

# Use a minimal base image for the final stage
FROM debian:12.7

WORKDIR /app
COPY --from=builder /app/target/release/backup .

# Install required packages in one go and clean up to reduce image size
RUN apt-get update && apt-get install -y \
    openssh-client \
    libssh-dev \
    curl \
    && curl -fsSL https://get.docker.com -o get-docker.sh \
    && sh get-docker.sh \
    && rm -rf /var/lib/apt/lists/* /get-docker.sh

# Run the binary
CMD ["./backup"]
