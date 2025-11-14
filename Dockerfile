# Build stage
FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev protobuf-dev

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./

# Copy proto files
COPY proto ./proto

# Copy source code
COPY src ./src

# Build release binary
RUN cargo build --release --bin toonify

# Runtime stage
FROM alpine:latest

RUN apk add --no-cache ca-certificates libgcc

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/toonify /usr/local/bin/toonify

# Create non-root user
RUN addgroup -g 1000 toonify && \
    adduser -D -u 1000 -G toonify toonify && \
    chown -R toonify:toonify /app

USER toonify

# Default to server mode, but allow override
ENTRYPOINT ["toonify"]
CMD ["serve"]

# Expose ports
EXPOSE 5000 50051

