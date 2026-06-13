#!/usr/bin/env bash
set -e

# Ensure we are in the project root
cd "$(dirname "$0")/.."

# 1. Run tests
echo "Running tests..."
./scripts/test.sh

# 2. Build release
echo "Building release..."
./scripts/build.sh

# 3. Tag a Git release
VERSION=$(grep -m 1 "^version" Cargo.toml | cut -d '"' -f 2)
echo "Tagging git release for version v$VERSION..."

# Check if the tag already exists
if git rev-parse "v$VERSION" >/dev/null 2>&1; then
    echo "Tag v$VERSION already exists. Recreating it..."
    git tag -d "v$VERSION"
fi

git tag -a "v$VERSION" -m "Release v$VERSION"

# 4. Push tags
echo "Pushing tag to origin..."
if git remote | grep -q "origin"; then
    git push origin "v$VERSION"
else
    echo "No remote 'origin' configured, skipping push."
fi

echo "Release v$VERSION tagged and pushed successfully!"
