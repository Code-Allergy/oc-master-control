# Stage 1: Build
FROM rust:latest as builder

# Install necessary packages and set up the build environment
RUN apt update && apt install -y \
    build-essential \
    libpq-dev \
    libssl-dev

# Create a new directory for the application
WORKDIR /usr/src/app

# Copy the source code into the container
COPY . .

# Build the application
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt update && apt install -y \
    libpq5

# Create a directory for the application
WORKDIR /usr/src/app

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/site /usr/src/app/

# Set the entrypoint to the compiled binary
ENTRYPOINT ["/usr/src/app/site"]

# Optionally, expose ports or add any other configuration
EXPOSE 3000

# Run the application
CMD ["site"]

