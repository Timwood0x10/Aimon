# 🚀 System Alert - Release Guide

Complete automation for building and releasing System Alert v0.2.1

## 📦 Available Release Methods

### 1. **Quick Release (Recommended)**
One command to rule them all:
```bash
./scripts/quick-release.sh
```
This will:
- ✅ Prompt for new version number
- ✅ Update Cargo.toml and Cargo.lock
- ✅ Build and test everything
- ✅ Commit and tag the release
- ✅ Push to GitHub (triggers automatic CI/CD)

### 2. **Manual Release Process**
Step by step control:
```bash
# Build release package
make package

# Test the release
make test-release

# Create GitHub release (requires gh CLI)
make github-release
```

### 3. **Individual Scripts**
For fine-grained control:
```bash
# Build optimized release
./scripts/build-release.sh

# Test the binary
./scripts/test-release.sh

# Create GitHub release
./scripts/github-release.sh
```

## 🛠 Available Make Targets

```bash
make help           # Show all available targets
make build          # Build debug version
make release        # Build optimized release
make test           # Run tests
make test-release   # Test release binary
make clean          # Clean build artifacts
make install        # Install locally (requires sudo)
make uninstall      # Remove from system
make package        # Create distribution package
make github-release # Create GitHub release
make all            # Build, test, and package
make dev-run        # Run development version
make release-run    # Run release version
```

## 🤖 Automated CI/CD

### GitHub Actions Workflows

1. **CI Workflow** (`.github/workflows/ci.yml`)
   - Triggers on: Push to main/develop, Pull Requests
   - Actions: Format check, Clippy, Tests, Build verification

2. **Release Workflow** (`.github/workflows/release.yml`)
   - Triggers on: Git tags (v*), Manual dispatch
   - Actions: Build, Test, Package, Create GitHub Release

### Automatic Release Process
When you push a tag like `v0.2.1`:
1. 🔨 GitHub Actions builds the release
2. 🧪 Runs all tests
3. 📦 Creates distribution packages
4. 🚀 Publishes GitHub release with assets
5. ✅ Ready for download!



## 📁 Release Artifacts

Each release creates:
```
dist/
├── system-alert-v0.2.1-macos.tar.gz     # Main distribution
└── system-alert-v0.2.1-checksums.txt    # SHA256 checksums

release-builds/
├── system-alert                          # Binary
├── README.txt                           # Installation guide
├── install.sh                           # Installation script
└── uninstall.sh                         # Uninstall script
```

## 🔐 Security

- All releases include SHA256 checksums
- Binaries are built in clean GitHub Actions environment
- No secrets or credentials in build process
- Reproducible builds with locked dependencies

## 🎯 Quick Start for Users

Users can install with:
```bash
# Download latest release
curl -L -o system-alert.tar.gz https://github.com/yourusername/system-alert/releases/latest/download/system-alert-v0.2.1-macos.tar.gz

# Extract and install
tar -xzf system-alert.tar.gz
cd system-alert-*
./install.sh

# Run
sudo system-alert
```

## 📊 Version 0.2.1 Features

✅ **Bilingual README** (English + Chinese)
✅ **Screenshot gallery** added to documentation
✅ **Apache License 2.0** licensing cleanup
✅ **Image reorganization** for layout screenshots
✅ **UI/component polishing** in startup and components renderer
✅ **Serial number masking** simplified for short identifiers
✅ **Dependency updates** via Cargo.lock refresh
✅ **Carbon/efficiency advisor** rendering continued
✅ **Headless output paths validated** (json/csv/prometheus)
✅ **Main async loop stabilized** after refactor
✅ **Release automation maintained** for CI/CD pipelines

---

## 🔧 Development Workflow

```bash
# Daily development
make dev            # Clean, build, test
make dev-run        # Run development version

# Pre-release testing
make release-cycle  # Full release build and test

# Release
./scripts/quick-release.sh  # One-command release
```

## 📋 Migration from 0.2.0

- No breaking API changes in this patch
- `config.toml` format remains compatible
- `--lang en|zh` and theme behavior unchanged
- Screenshots now available under `images/`

## 🐛 Fixed in 0.2.1

- Startup page serial masking logic for length <= 4
- Theme/image asset naming consistency
- Docs and license alignment
- CI artifact references updated

---

🎉 **Ready to release System Alert v0.2.1!**

Choose your preferred method and let the automation handle the rest!
