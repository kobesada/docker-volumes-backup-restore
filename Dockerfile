FROM rust:latest AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
RUN cargo build --release

FROM ubuntu:24.04
WORKDIR /app
COPY --from=builder /app/target/release/backup .

RUN apt-get update
RUN apt-get install -y openssh-client docker.io

CMD ["./backup"]
