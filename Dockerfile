FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
RUN mkdir -p /app/workspace/.unthinkclaw
COPY target/x86_64-unknown-linux-gnu/release/unthinkclaw .
COPY container-config.json /app/unthinkclaw.json
EXPOSE 8080
ENTRYPOINT ["./unthinkclaw", "mcp", "--port", "8080", "--config", "/app/unthinkclaw.json", "--workspace", "/app/workspace"]
