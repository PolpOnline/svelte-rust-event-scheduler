# Dockerfile

# Use the official Rust image from the Docker Hub
FROM rust:latest as builder

# Create a new directory in the container
WORKDIR /usr/src/app

# Copy the project's source code to the WORKDIR
COPY . .

RUN apt update && \
    apt install -y protobuf-compiler

# Build the Rust project
RUN cargo install --profile release --path .

# Start a new build stage - this is where the application will be run
FROM debian:latest

# Copy the built application from the "builder" stage
COPY --from=builder /usr/src/app/target/release/svelte-rust-event-scheduler /usr/local/bin

# Expose port 8000
EXPOSE 8000

# Start the application
CMD ["svelte-rust-event-scheduler"]