# Build stage
FROM rust:1.82-slim-bookworm as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    clang \
    lld \
    llvm \
    libclang-dev \
    pkg-config \
    build-essential \
    libgstreamer1.0-dev \
    libgstreamer-plugins-base1.0-dev \
    gstreamer1.0-tools \
    gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad \
    gstreamer1.0-plugins-ugly \
    libopencv-dev \
    libopencv-core-dev \
    libopencv-imgproc-dev \
    v4l2loopback-dkms \
    v4l-utils \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install wasm-pack
RUN cargo install wasm-pack

# Set LIBCLANG_PATH for clang-sys
ENV LIBCLANG_PATH=/usr/lib/llvm-14/lib

# Set working directory
WORKDIR /app

# Copy Cargo manifests
COPY Cargo.toml .
COPY server/Cargo.toml server/
COPY client/Cargo.toml client/

# Create dummy src files to cache dependency builds
RUN mkdir -p server/src client/src
RUN echo "fn main() {}" > server/src/main.rs
RUN echo "" > client/src/lib.rs

# Pre-build to cache dependencies
RUN cargo build --release || true

# Copy actual source code
COPY . .

# Build client WebAssembly package
WORKDIR /app/client
RUN wasm-pack build --target web --out-dir ../static/pkg

# Build server binary
WORKDIR /app/server
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime system dependencies
RUN apt-get update && apt-get install -y \
    libopencv-core-dev \
    libopencv-imgproc-dev \
    v4l-utils \
    libgstreamer1.0-0 \
    libgstreamer-plugins-base1.0-0 \
    && rm -rf /var/lib/apt/lists/*

# Copy built server binary and static files
WORKDIR /usr/local/bin
COPY --from=builder /app/target/release/server .

WORKDIR /usr/local/static
COPY --from=builder /app/static .

# Expose Rocket port
EXPOSE 8000

# Set default command
CMD ["/usr/local/bin/server"]
