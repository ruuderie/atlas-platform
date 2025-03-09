FROM rust:1.84.1 AS builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the entire project
COPY . .

# Build the application
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/business_directory_backend .

# Create a non-root user to run the application
RUN useradd -m appuser
USER appuser

# Set environment variables
ENV RUST_LOG=info

# Expose the port
EXPOSE 8000

# Run the application
CMD ["./business_directory_backend"] 