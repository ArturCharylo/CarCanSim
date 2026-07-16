# Stage 1: Build environment
FROM rust:slim-bookworm AS builder

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release

# Stage 2: Runtime environment
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl3 && rm -rf /var/lib/apt/lists/*

# Copy only the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/CarCanSim /usr/local/bin/CarCanSim

EXPOSE 8080

CMD ["CarCanSim"]