FROM debian:bookworm-slim

# Install CA certificates aja (buat HTTPS kalo ada)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary
COPY target/release/game-matrix /app/game-matrix
RUN chmod +x /app/game-matrix

# Expose port
EXPOSE 3001

CMD ["/app/game-matrix"]