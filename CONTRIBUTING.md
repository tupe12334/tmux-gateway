# Contributing to tmux-gateway

Thanks for your interest in contributing! This guide will help you get started.

## Development Guidelines

- Make sure to work in DDD and functional core imperative shell style, i.e. the api network can be rest, grpc or graphql but the issue should be focused on the domain and not the api, so every task is api agnostic and relevant to all existing apis
- Prefer usage of existing libs rather creating custom logic and implementations, for example, if you need to parse a yaml file, use an existing yaml parsing library instead of writing your own parser
- Write tests for your changes to ensure they work as expected and to prevent regressions in the future
- Make sure files are small and focused on a single responsibility, if a file is getting too big, consider splitting it into smaller files

## Releasing

Releases are fully automated via GitHub Actions. Pushing a tag matching `v*` triggers the release workflow (`.github/workflows/release.yml`), which:

1. Runs clippy and the full test suite
2. Cross-compiles binaries for linux (x86_64, aarch64) and macOS (x86_64, aarch64)
3. Packages each binary into a `.tar.gz` archive with a SHA256 checksum file
4. Creates a GitHub Release with auto-generated release notes and uploads all artifacts

### How to cut a release

```bash
# Tag the commit (use semver)
git tag v0.1.0

# Push the tag to trigger the workflow
git push origin v0.1.0
```

### Verifying a downloaded artifact

```bash
shasum -a 256 -c tmux-gateway-v0.1.0-x86_64-unknown-linux-gnu.tar.gz.sha256
```
