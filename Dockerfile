
# Base image with Rust
#FROM rust:latest as builder
# Set the working directory
#WORKDIR /app
#COPY . .
#RUN cargo build --release

# stage 2

#FROM rust:latest
#RUN apt-get update && apt-get install libpq-dev -y

#WORKDIR /app
#COPY --from=builder /app/target/release/site /usr/bin/site
#COPY --from=builder /app/static .

#CMD ["/usr/bin/site"]

# Stage 1: Build
FROM debian:bullseye-slim as builder

# Install necessary packages and set up the build environment
RUN apt update && apt install \
    build-base \
    postgresql-dev \
    openssl-dev

# Create a new directory for the application
WORKDIR /usr/src/app

# Copy the source code into the container
COPY . .

# Build the application
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bullseye-slim

# Install necessary packages for the runtime environment
RUN apt update && apt add --no-cache \
    libgcc \
    libstdc++ \
    postgresql-dev

# Create a directory for the application
WORKDIR /usr/src/app

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/site /usr/src/app/

# Set the entrypoint to the compiled binary
ENTRYPOINT ["/usr/src/app/site"]

# Optionally, expose ports or add any other configuration
# EXPOSE 3000

# Run the application
CMD ["site"]

