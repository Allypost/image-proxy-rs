FROM rust:1.73-bullseye as builder
RUN apt-get update && apt-get install -y musl-tools musl-dev upx-ucl
WORKDIR /app
COPY . .
RUN cargo build --release --locked
RUN upx --best --lzma /app/target/release/image-proxy

FROM debian:bullseye-slim as runner
# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/image-proxy /bin/image-proxy
CMD ["image-proxy"]
