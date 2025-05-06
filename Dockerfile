# ---- Stage 1: Build ----
# Use the official Rust image as the base for building
FROM rust:1.78 AS builder

# Set the working directory inside the container
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files first to leverage Docker cache
COPY Cargo.toml Cargo.lock ./

# Create a dummy src/main.rs to build dependencies only (faster rebuilds)
RUN mkdir src && echo "fn main(){}" > src/main.rs
RUN cargo build --release --locked

# Now copy the actual source code
COPY src ./src

# Build the application in release mode
# Ensure necessary build tools are available if using specific libraries
# (e.g., musl for smaller static builds, but requires target setup)
# Clean previous dummy build artifacts before the real build
RUN touch src/main.rs && cargo build --release --locked

# ---- Stage 2: Run ----
# Use a minimal base image for the final container
# Using Debian Slim for glibc compatibility which is often easier than musl/Alpine
FROM debian:bullseye-slim

# Set the working directory
WORKDIR /app

# Install runtime dependencies if any (e.g., OpenSSL if needed, but axum/tokio often don't require it explicitly)
# RUN apt-get update && apt-get install -y --no-install-recommends openssl ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/plant_care_app .

# Copy static files and create necessary directories
COPY static ./static
COPY uploads ./uploads
COPY data ./data

# Create directories if they might not exist (especially important for volume mounts later)
# Ensure the application user can write to data and uploads
RUN mkdir -p /app/data && mkdir -p /app/uploads && chown -R 1000:1000 /app/data /app/uploads /app/static
# (Using a non-root user is good practice, but requires USER directive and potentially permission adjustments)
# USER 1000

# Expose the port the application listens on (must match main.rs)
EXPOSE 3000

# Define the command to run the application
# The application binary is now in the WORKDIR /app
CMD ["./plant_care_app"]