# syntax=docker/dockerfile:1

# -----------------------------------------------------------------------------
# Base Stage: Common Environment
# -----------------------------------------------------------------------------
FROM rust:1-bookworm AS base
WORKDIR /app

# Install system dependencies
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    curl \
    file \
    git \
    libayatana-appindicator3-dev \
    libgtk-3-dev \
    librsvg2-dev \
    libssl-dev \
    libudev-dev \
    libusb-1.0-0-dev \
    libwebkit2gtk-4.1-dev \
    pkg-config \
    udev \
    wget \
    && curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && npm install -g yarn pnpm

# Install cargo-chef for dependency caching
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo install cargo-chef cargo-watch

# -----------------------------------------------------------------------------
# Development Stage (Hot Reload + Debugging)
# -----------------------------------------------------------------------------
FROM base AS dev

# Development environment variables
ENV TAURI_ENV_PLATFORM=linux \
    TAURI_ENV_ARCH=x86_64 \
    TAURI_ENV_DEBUG=true \
    RUST_BACKTRACE=1 \
    RUST_LOG=debug

# Install development tools
RUN --mount=type=cache,target=/root/.npm \
    --mount=type=cache,target=/usr/local/cargo/registry \
    npm install -g @tauri-apps/cli && \
    cargo install cargo-expand cargo-edit cargo-outdated

# Install udev rules for USB hardware access
COPY --chown=root:root <<'EOF' /etc/udev/rules.d/99-android-tools.rules
# Amlogic devices
SUBSYSTEM=="usb", ATTR{idVendor}=="1b8e", MODE="0666", GROUP="plugdev"
# Rockchip devices
SUBSYSTEM=="usb", ATTR{idVendor}=="2207", MODE="0666", GROUP="plugdev"
# Allwinner devices
SUBSYSTEM=="usb", ATTR{idVendor}=="1f3a", MODE="0666", GROUP="plugdev"
# CH340 UART
SUBSYSTEM=="tty", ATTRS{idVendor}=="1a86", ATTRS{idProduct}=="7523", MODE="0666", GROUP="plugdev"
# CP2102 UART
SUBSYSTEM=="tty", ATTRS{idVendor}=="10c4", ATTRS{idProduct}=="ea60", MODE="0666", GROUP="plugdev"
EOF

# Create non-root user for development
RUN groupadd -g 1000 developer && \
    useradd -m -u 1000 -g developer -G plugdev developer && \
    mkdir -p /home/developer/.cargo /home/developer/.npm && \
    chown -R developer:developer /home/developer

USER developer
WORKDIR /app

# Pre-install common dependencies (cached layer)
COPY --chown=developer:developer Cargo.toml Cargo.lock ./
USER root
RUN mkdir -p phoenix-cli/src phoenix-lib/src ui/src-tauri/src && \
    echo "fn main() {}" > phoenix-cli/src/main.rs && \
    echo "fn main() {}" > phoenix-lib/src/lib.rs && \
    echo "fn main() {}" > ui/src-tauri/src/main.rs && \
    chown -R developer:developer /app

USER developer

# Development startup script
COPY --chown=developer:developer <<'EOF' /usr/local/bin/dev-start.sh
#!/bin/bash
set -e

echo "🚀 Phoenix Development Environment"
echo "=================================="
echo ""
echo "Available commands:"
echo "  cargo watch -x 'run --bin phoenix'    # Run CLI with hot reload"
echo "  cd ui && npm run tauri dev            # Run GUI with hot reload"
echo "  cargo test                             # Run tests"
echo "  cargo fmt && cargo clippy             # Format and lint"
echo ""

exec "$@"
EOF

RUN chmod +x /usr/local/bin/dev-start.sh

# Default command for development
CMD ["/usr/local/bin/dev-start.sh", "bash"]

# -----------------------------------------------------------------------------
# Planner Stage (Dependency Analysis)
# -----------------------------------------------------------------------------
FROM base AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# -----------------------------------------------------------------------------
# Cacher Stage (Dependency Building)
# -----------------------------------------------------------------------------
FROM base AS cacher
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json

# -----------------------------------------------------------------------------
# Frontend Builder Stage
# -----------------------------------------------------------------------------
FROM base AS frontend-builder
WORKDIR /app/ui

# Copy frontend dependencies
COPY ui/package.json ui/package-lock.json* ui/yarn.lock* ui/pnpm-lock.yaml* ./

# Install dependencies with cache
RUN --mount=type=cache,target=/root/.npm \
    if [ -f pnpm-lock.yaml ]; then \
      pnpm install --frozen-lockfile; \
    elif [ -f yarn.lock ]; then \
      yarn install --frozen-lockfile; \
    else \
      npm ci; \
    fi

# Copy frontend source
COPY ui/ ./

# Build frontend
RUN --mount=type=cache,target=/app/ui/node_modules/.cache \
    npm run build

# -----------------------------------------------------------------------------
# Backend Builder Stage
# -----------------------------------------------------------------------------
FROM base AS backend-builder
WORKDIR /app

# Copy cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo

# Copy source code
COPY Cargo.toml Cargo.lock ./
COPY phoenix-cli/ phoenix-cli/
COPY phoenix-lib/ phoenix-lib/
COPY ui/src-tauri/ ui/src-tauri/

# Copy built frontend
COPY --from=frontend-builder /app/ui/dist ui/dist/

# Build release binaries
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --workspace && \
    mkdir -p /tmp/bin && \
    cp target/release/phoenix /tmp/bin/phoenix && \
    cp target/release/phoenix-gui-app /tmp/bin/phoenix-gui && \
    strip /tmp/bin/phoenix /tmp/bin/phoenix-gui

# -----------------------------------------------------------------------------
# Production Stage (Minimal Runtime)
# -----------------------------------------------------------------------------
FROM debian:bookworm-slim AS prod

# Install only runtime dependencies
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libayatana-appindicator3-1 \
    libgtk-3-0 \
    libssl3 \
    libudev1 \
    libusb-1.0-0 \
    libwebkit2gtk-4.1-0 \
    udev && \
    apt-get clean

# Create non-root user
RUN groupadd -g 1000 phoenix && \
    useradd -m -u 1000 -g phoenix phoenix

# Install udev rules
COPY --chown=root:root <<'EOF' /etc/udev/rules.d/99-android-tools.rules
SUBSYSTEM=="usb", ATTR{idVendor}=="1b8e", MODE="0666", GROUP="phoenix"
SUBSYSTEM=="usb", ATTR{idVendor}=="2207", MODE="0666", GROUP="phoenix"
SUBSYSTEM=="usb", ATTR{idVendor}=="1f3a", MODE="0666", GROUP="phoenix"
SUBSYSTEM=="tty", ATTRS{idVendor}=="1a86", ATTRS{idProduct}=="7523", MODE="0666", GROUP="phoenix"
SUBSYSTEM=="tty", ATTRS{idVendor}=="10c4", ATTRS{idProduct}=="ea60", MODE="0666", GROUP="phoenix"
EOF

WORKDIR /app

# Copy binaries
COPY --from=backend-builder --chown=phoenix:phoenix /tmp/bin/phoenix /usr/local/bin/phoenix
COPY --from=backend-builder --chown=phoenix:phoenix /tmp/bin/phoenix-gui /usr/local/bin/phoenix-gui

# Copy configs and resources
COPY --chown=phoenix:phoenix configs/ /app/configs/

USER phoenix

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD ["phoenix", "detect", "--json"] || exit 1

CMD ["phoenix"]

# -----------------------------------------------------------------------------
# CLI-Only Stage (Smaller Image for Headless Servers)
# -----------------------------------------------------------------------------
FROM debian:bookworm-slim AS cli

RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    libudev1 \
    libusb-1.0-0 \
    udev && \
    apt-get clean

RUN groupadd -g 1000 phoenix && \
    useradd -m -u 1000 -g phoenix phoenix

COPY --chown=root:root <<'EOF' /etc/udev/rules.d/99-android-tools.rules
SUBSYSTEM=="usb", ATTR{idVendor}=="1b8e", MODE="0666", GROUP="phoenix"
SUBSYSTEM=="usb", ATTR{idVendor}=="2207", MODE="0666", GROUP="phoenix"
SUBSYSTEM=="usb", ATTR{idVendor}=="1f3a", MODE="0666", GROUP="phoenix"
SUBSYSTEM=="tty", ATTRS{idVendor}=="1a86", MODE="0666", GROUP="phoenix"
SUBSYSTEM=="tty", ATTRS{idVendor}=="10c4", MODE="0666", GROUP="phoenix"
EOF

WORKDIR /app

COPY --from=backend-builder --chown=phoenix:phoenix /tmp/bin/phoenix /usr/local/bin/phoenix
COPY --chown=phoenix:phoenix configs/ /app/configs/

USER phoenix

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD ["phoenix", "detect", "--json"] || exit 1

ENTRYPOINT ["phoenix"]
CMD ["--help"]
