# ============================================================================
# Build Arguments (configurable settings)
# ============================================================================
ARG RUST_VERSION=1.90
ARG DEBIAN_VERSION=bookworm
ARG CHEF_VERSION=0.1.70
ARG WASM_SOURCE_DIR=./packages/contracts/artifacts
ARG WASM_TARGET_DIR=/wasm/built-in
ARG CLI_PACKAGE=wavs-cosmos-cli

# ============================================================================
# Stage 1: Chef Planner
# ============================================================================
FROM rust:${RUST_VERSION}-slim-${DEBIAN_VERSION} AS chef

ARG CHEF_VERSION

# Install cargo-chef for dependency caching
RUN cargo install cargo-chef --version ${CHEF_VERSION} --locked

WORKDIR /app

# ============================================================================
# Stage 2: Prepare Recipe
# ============================================================================
FROM chef AS planner

# Copy workspace files needed for planning
COPY Cargo.toml Cargo.lock ./
COPY packages ./packages

# Generate dependency recipe
RUN cargo chef prepare --recipe-path recipe.json

# ============================================================================
# Stage 3: Build Dependencies (Cached Layer)
# ============================================================================
FROM chef AS cacher

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the recipe from planner
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies - this layer will be cached unless dependencies change
RUN cargo chef cook --release --recipe-path recipe.json

# ============================================================================
# Stage 4: Build Application
# ============================================================================
FROM rust:${RUST_VERSION}-slim-${DEBIAN_VERSION} AS builder

ARG CLI_PACKAGE

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy cached dependencies from cacher stage
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo

# Copy source code
COPY Cargo.toml Cargo.lock ./
COPY packages ./packages

# Build the CLI binary
RUN cargo build --release --package ${CLI_PACKAGE}

# ============================================================================
# Stage 5: Runtime Image
# ============================================================================
FROM debian:${DEBIAN_VERSION}-slim AS runtime

ARG WASM_SOURCE_DIR
ARG WASM_TARGET_DIR
ARG CLI_PACKAGE

# Install runtime dependencies for TLS/HTTPS
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create wasm directory
RUN mkdir -p ${WASM_TARGET_DIR}

# Copy the compiled binary from builder
COPY --from=builder /app/target/release/${CLI_PACKAGE} /usr/local/bin/cli

# Copy WASM files from host
COPY ${WASM_SOURCE_DIR}/*.wasm ${WASM_TARGET_DIR}/

# Set the binary as executable (should already be, but explicit is good)
RUN chmod +x /usr/local/bin/cli

# Set working directory
WORKDIR /app

# Default entrypoint - passes through all arguments
ENTRYPOINT ["/usr/local/bin/cli"]

# Default command (can be overridden)
CMD ["--help"]
