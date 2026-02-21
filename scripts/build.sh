#!/usr/bin/env bash
set -e

# Get git commit hash
GIT_HASH=$(git rev-parse --short HEAD)

# Set defaults
REGISTRY="${REGISTRY:-docker.io}"
ORG="${REGISTRY_ORG:-library}"

# Calculate image names
BACKEND_IMAGE="${REGISTRY}/${ORG}/oceaniam:${GIT_HASH}"
MIGRATION_IMAGE="${REGISTRY}/${ORG}/oceaniam-migration:${GIT_HASH}"

echo "Building backend image: ${BACKEND_IMAGE}"
docker build --target backend -t "${BACKEND_IMAGE}" .

echo "Building migration image: ${MIGRATION_IMAGE}"
docker build --target migration -t "${MIGRATION_IMAGE}" .

echo "Build completed successfully!"
echo "Backend: ${BACKEND_IMAGE}"
echo "Migration: ${MIGRATION_IMAGE}"
