# Multi-stage build for Rust application
# Stage 1: Build stage
FROM rust:1.75-slim-bullseye as builder

# Install system dependencies needed for compilation
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Cargo files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached unless Cargo.toml changes)
RUN cargo build --release
RUN rm src/main.rs

# Copy source code
COPY src/ ./src/

# Build the actual application
RUN cargo build --release

# Stage 2: Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN useradd -r -s /bin/false mockservice

# Set working directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/mock-service /app/mock-service

# Copy services directory for mock data
COPY services/ ./services/

# Change ownership to non-root user
RUN chown -R mockservice:mockservice /app

# Switch to non-root user
USER mockservice

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/healthz || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV HOST=0.0.0.0
ENV PORT=8080

# Run the application
CMD ["./mock-service", "--host", "0.0.0.0", "--port", "8080"]