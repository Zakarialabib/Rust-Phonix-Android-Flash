# syntax=docker/dockerfile:1

# 1. Planner Stage: Compute recipe for caching
FROM rust:1-bookworm AS planner
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# 2. Cacher Stage: Cache dependencies
FROM rust:1-bookworm AS cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
# Install system dependencies required for sys-crates (e.g. rusb/libusb)
RUN apt-get update && apt-get install -y \
    libusb-1.0-0-dev \
    libudev-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*
RUN cargo chef cook --release --recipe-path recipe.json

# 3. Builder Stage: Build the project
FROM rust:1-bookworm AS builder
WORKDIR /app

# Install dependencies for Tauri and low-level hardware
RUN apt-get update && apt-get install -y \
    libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libusb-1.0-0-dev \
    libudev-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js for frontend build
RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash - && \
    apt-get install -y nodejs

# Copy cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo

# Copy source code
COPY . .

# Build frontend
WORKDIR /app/ui
RUN npm install
RUN npm run build

# Build backend/CLI
WORKDIR /app
RUN cargo build --release --workspace

# 4. Test Stage
FROM builder AS tester
CMD ["cargo", "test", "--workspace", "--verbose"]
