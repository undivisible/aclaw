FROM rust:1.82-slim AS builder
RUN apt-get update && apt-get install -y pkg-config libclang-dev clang && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/unthinkclaw .
EXPOSE 8080
ENTRYPOINT ["./unthinkclaw", "mcp", "--port", "8080"]
