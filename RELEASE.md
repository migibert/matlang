# Release Process

This document describes how to create releases with automated binary builds for multiple platforms.

## Automated Releases

The project uses GitHub Actions to automatically build optimized binaries for:
- Linux x86_64
- Linux ARM64 (aarch64)
- macOS x86_64 (Intel)
- macOS ARM64 (Apple Silicon)
- Windows x86_64

## Creating a Release

1. **Update version in Cargo.toml** (if needed):
   ```toml
   [package]
   version = "0.2.0"
   ```

2. **Create and push a version tag**:
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

3. **GitHub Actions will automatically**:
   - Build binaries for all platforms
   - Strip debug symbols (Linux/macOS)
   - Create compressed archives (.tar.gz for Unix, .zip for Windows)
   - Create a GitHub release with all binaries attached
   - Generate release notes from commits

4. **The release will appear** at:
   ```
   https://github.com/YOUR_USERNAME/martial-lang/releases
   ```

## Manual Testing Before Release

Test locally before creating a tag:

```bash
# Build release binary
make release

# Run all tests
make test

# Test on example systems
mat validate examples/bjj-basic
mat validate examples/muay-thai-basic
mat stats examples/bjj-basic
mat dot examples/muay-thai-basic | dot -Tpng > test.png
```

## Manual Release Build

If you need to manually trigger a release build:

1. Go to the Actions tab on GitHub
2. Select "Release" workflow
3. Click "Run workflow"
4. Select the branch/tag to build from

## Workflow Configuration

The release workflow is defined in `.github/workflows/release.yml` and includes:

- **Cross-compilation**: Builds for multiple architectures
- **Binary stripping**: Removes debug symbols to reduce size
- **Artifact upload**: Makes binaries available in Actions
- **Release creation**: Automatically creates GitHub releases for tags

## Local Multi-Platform Build

To build for a specific platform locally:

```bash
# Install target
rustup target add aarch64-unknown-linux-gnu

# Build
cargo build --release --target aarch64-unknown-linux-gnu

# Binary at: target/aarch64-unknown-linux-gnu/release/mat
```

## Binary Size Optimization

Current release binary size: ~597K (Linux x86_64)

Already optimized via:
- Rust's release profile (`cargo build --release`)
- Symbol stripping (`strip` command)
- Minimal dependencies (only serde + serde_json)

Further optimization is possible but would provide diminishing returns.
