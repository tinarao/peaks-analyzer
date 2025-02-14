# Stage 1: Build the application
FROM rustlang/rust:nightly as builder

# Install necessary dependencies for building the application
RUN apt-get update && apt-get install -y \
    libasound2-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Create a new empty shell project
RUN USER=root cargo new --bin rust-peaks-analyzer
WORKDIR /rust-peaks-analyzer

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src

# Copy the .env file
COPY .env .env

COPY db db

# Build the application in release mode
RUN cargo +nightly build --release

# Stage 2: Create the runtime image
FROM ubuntu:22.04

# Install necessary dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    libasound2-dev \
    pkg-config \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /rust-peaks-analyzer/target/release/rust-peaks-analyzer /usr/local/bin/rust-peaks-analyzer

# Copy the .env file
COPY .env /usr/local/bin/.env

# Ensure the binary has execution permissions
RUN chmod +x /usr/local/bin/rust-peaks-analyzer

# Set the entrypoint to the compiled binary
ENTRYPOINT ["/usr/local/bin/rust-peaks-analyzer"]

# Expose the port that the web server will run on
EXPOSE 4200