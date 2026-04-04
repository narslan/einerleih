#!/usr/bin/env bash
set -euo pipefail

IMAGE_REF="${1:-ghcr.io/narslan/einerleih:latest}"

if [[ ! -f static/dist/index.html ]]; then
  echo "Fehlende Frontend-Dateien: static/dist/index.html wurde nicht gefunden." >&2
  echo "Baue zuerst das Frontend und lege die gebauten Dateien unter static/dist ab." >&2
  exit 1
fi

cargo build --release
docker build -t "${IMAGE_REF}" .

echo "Built image: ${IMAGE_REF}"
