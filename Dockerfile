FROM rust:latest AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
RUN cargo build --release

FROM backup-runtime
WORKDIR /app
COPY --from=builder /app/target/release/backup .

# Run the binary
CMD ["./backup"]
