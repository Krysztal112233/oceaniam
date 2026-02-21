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

echo "Pushing backend image: ${BACKEND_IMAGE}"
docker push "${BACKEND_IMAGE}"

echo "Pushing migration image: ${MIGRATION_IMAGE}"
docker push "${MIGRATION_IMAGE}"

echo "Push completed successfully!"
echo "Backend: ${BACKEND_IMAGE}"
echo "Migration: ${MIGRATION_IMAGE}"
