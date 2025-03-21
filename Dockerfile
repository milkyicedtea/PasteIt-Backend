# Build
FROM lukemathwalker/cargo-chef:latest-rust-1 as chef
WORKDIR /app

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release

# Create smaller image for exec
FROM ubuntu:24.04
WORKDIR /app

# Install required runtime
RUN apt-get update && apt-get install -y \
    libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create secrets directory
RUN mkdir -p /run/secrets

# Copy the compiled rust binary
COPY --from=builder /app/target/release/PasteIt-Backend /app/PasteIt-Backend

# Create debug script
RUN echo '#!/bin/bash\n\
echo "Contents of /run/secrets:"\n\
ls -la /run/secrets/\n\
echo "Starting application..."\n\
RUST_BACKTRACE=1 /app/PasteIt-Backend' > /app/start.sh

RUN chmod +x /app/start.sh
ENTRYPOINT ["/app/start.sh"]