# Use Rust official image as base
FROM rust:latest

# Install dependencies
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
    v4l-utils

# Set the environment variable for clang-sys
ENV LIBCLANG_PATH=/usr/lib/llvm-14/lib

# Set working directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Create a dummy src directory for caching
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies first for caching
RUN cargo build --release || true

# Copy actual source code
COPY . .

# Build final application
RUN cargo build --release

# Run the application
CMD ["./target/release/vex-vision-scoring"]
