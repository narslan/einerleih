FROM debian:trixie-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY target/release/einerleih /app/einerleih
COPY db /app/db
COPY static /app/static

RUN mkdir -p /app/assets/private/foto \
    && chmod +x /app/einerleih

ENV LISTEN=0.0.0.0:8000 \
    ASSETS_PUBLIC_PATH=/app/static \
    ASSETS_PUBLIC_URL=/static \
    ASSETS_PRIVATE_PATH=/app/assets/private \
    ASSETS_PRIVATE_URL=/file \
    ASSET_MAX_SIZE=5242880

EXPOSE 8000

CMD ["/app/einerleih"]
