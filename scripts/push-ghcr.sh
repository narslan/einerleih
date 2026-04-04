#!/usr/bin/env bash
set -euo pipefail

IMAGE_REF="${1:-ghcr.io/narslan/einerleih:latest}"

docker push "${IMAGE_REF}"

echo "Pushed image: ${IMAGE_REF}"
