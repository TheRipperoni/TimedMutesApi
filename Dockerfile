# Use a Rust base image with Cargo installed
FROM rust:latest AS builder

# Set the working server inside the container
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml /app
COPY Cargo.lock /app
# Now copy the source code
COPY ./src /app/src

# Build your application
RUN cargo build --release

# Start a new stage to create a smaller image without unnecessary build dependencies
FROM rust:slim

RUN apt-get update && apt-get install sqlite3 -y

# Copy the built binary from the previous stage
COPY --from=builder /app/target/release/TimedMutes .

# Copy the entrypoint script
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Command to run the application
ENTRYPOINT ["/bin/sh", "/entrypoint.sh"]
