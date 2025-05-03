# Use a lightweight base image for Raspberry Pi simulation
FROM debian

# Set environment variables
ENV DEBIAN_FRONTEND=noninteractive \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

# Install required dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    build-essential \
    gcc \
    libssl-dev \
    pkg-config \
    ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Install Rust using rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable

# Set the working directory
WORKDIR /app

# Copy the project files
COPY . .

# Build the Rust project
RUN cargo build --release

# Set the default command
CMD ["./target/debug/rin_agent.exe"]