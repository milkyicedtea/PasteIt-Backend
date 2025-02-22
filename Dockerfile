# Build
FROM rust:latest as builder
WORKDIR /app

# Copy dependencies
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

# Copy project and build it
COPY . .
RUN cargo build --release

# Create smaller image for exec
FROM ubuntu:24.04
WORKDIR /app

# Install required runtime
RUN apt-get update && apt-get install -y \
    libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the compiled rust binary
COPY --from=builder /app/target/release/PasteIt-Backend /app/PasteIt-Backend