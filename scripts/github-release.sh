#!/bin/bash

# Aimon - GitHub Release Script
# Creates a GitHub release with assets

set -e

echo "🐙 Aimon - GitHub Release Script"
echo "======================================="

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
PROJECT_NAME="aimon"
VERSION=$(grep '^version' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
REMOTE_URL=$(git config --get remote.origin.url || git remote get-url os 2>/dev/null || git remote get-url pd 2>/dev/null || true)
REPO_PATH=$(echo "${REMOTE_URL}" | sed -E 's#.*github.com[-_a-zA-Z0-9]*[:/]([^/]+/[^/.]+)(\.git)?#\1#')
REPO_OWNER=$(echo "${REPO_PATH}" | cut -d/ -f1)
REPO_NAME=$(echo "${REPO_PATH}" | cut -d/ -f2)

if [ -z "${REPO_OWNER}" ] || [ -z "${REPO_NAME}" ]; then
    echo -e "${RED}❌ Could not detect GitHub repository from git remotes${NC}"
    exit 1
fi
DIST_DIR="dist"

echo -e "${BLUE}📦 Preparing GitHub release for v${VERSION}${NC}"

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo -e "${RED}❌ GitHub CLI (gh) is not installed${NC}"
    echo -e "${YELLOW}Install it with: brew install gh${NC}"
    exit 1
fi

# Check if user is authenticated
if ! gh auth status &> /dev/null; then
    echo -e "${RED}❌ Not authenticated with GitHub${NC}"
    echo -e "${YELLOW}Run: gh auth login${NC}"
    exit 1
fi

# Check if distribution files exist
ARCHIVE_FILE="${DIST_DIR}/${PROJECT_NAME}-v${VERSION}-macos.tar.gz"
CHECKSUM_FILE="${DIST_DIR}/${PROJECT_NAME}-v${VERSION}-checksums.txt"

if [ ! -f "${ARCHIVE_FILE}" ]; then
    echo -e "${RED}❌ Distribution archive not found: ${ARCHIVE_FILE}${NC}"
    echo -e "${YELLOW}Run build-release.sh first${NC}"
    exit 1
fi

if [ ! -f "${CHECKSUM_FILE}" ]; then
    echo -e "${RED}❌ Checksum file not found: ${CHECKSUM_FILE}${NC}"
    echo -e "${YELLOW}Run build-release.sh first${NC}"
    exit 1
fi

RELEASE_NOTES_FILE="RELEASE.md"
if [ ! -s "${RELEASE_NOTES_FILE}" ]; then
    echo -e "${RED}❌ Release notes file not found: ${RELEASE_NOTES_FILE}${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Using release notes from ${RELEASE_NOTES_FILE}${NC}"

# Create the release
echo -e "${YELLOW}🚀 Creating GitHub release...${NC}"

gh release create "v${VERSION}" \
    "${ARCHIVE_FILE}" \
    "${CHECKSUM_FILE}" \
    --title "Aimon v${VERSION}" \
    --notes-file "${RELEASE_NOTES_FILE}" \
    --draft

echo -e "${GREEN}🎉 GitHub release created successfully!${NC}"
echo -e "${BLUE}📋 Next steps:${NC}"
echo -e "   1. Review the draft release at: https://github.com/${REPO_OWNER}/${REPO_NAME}/releases"
echo -e "   2. Edit release notes if needed"
echo -e "   3. Publish the release when ready"

echo -e "${GREEN}✨ Release process complete!${NC}"
