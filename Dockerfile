FROM rust:1.75 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/app/target/release/hcp-gateway /usr/local/bin/hcp-gateway
COPY --from=builder /usr/src/app/data /usr/local/bin/data
WORKDIR /usr/local/bin
EXPOSE 8080
CMD ["hcp-gateway"]
