# Base image with Rust
FROM ubuntu:latest

RUN apt-get update && apt-get upgrade -y && \
    apt-get install -y \
    build-essential \
    libpq-dev \
    nodejs \
    npm \
    curl \
    gnupg

# Install Rust and Cargo
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    export PATH="$PATH:/root/.cargo/bin" && \
    rustup update

# Set up Node.js to the latest LTS version
RUN curl -fsSL https://deb.nodesource.com/setup_lts.x | bash - && \
    apt-get install -y nodejs

# Set the working directory

EXPOSE 3000

WORKDIR /app

COPY . .
