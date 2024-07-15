# Use the official Rust image as the build stage
FROM rust:1.79.0 as builder

# Set the working directory
WORKDIR /usr/src/pricing_application

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src

# Build the application
RUN cargo build --release -Z sparse-registry

# Use the official Debian image for the final stage
FROM debian:buster-slim

# Copy the compiled binary from the build stage
COPY --from=builder /usr/src/pricing_application/target/release/pricing_rust /usr/local/bin/pricing_rust

# Set the startup command
CMD ["pricing_rust"]

# Expose the port the app runs on
EXPOSE 8000
