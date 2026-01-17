# Publishing to crates.io

This document describes how to publish `tuiw` to crates.io.

## Prerequisites

1. Create an account on [crates.io](https://crates.io/)
2. Login via GitHub
3. Get your API token from [https://crates.io/me](https://crates.io/me)

## Setup

```bash
cargo login <your-api-token>
```

## Publishing Steps

### 1. Update Version

Edit `Cargo.toml` and bump the version:

```toml
[package]
version = "0.1.1"  # Increment as needed
```

### 2. Update CHANGELOG

Document changes in the new version.

### 3. Commit Changes

```bash
git add Cargo.toml CHANGELOG.md
git commit -m "bump version to 0.1.1"
```

### 4. Create Git Tag

```bash
git tag -a v0.1.1 -m "Release v0.1.1"
git push origin v0.1.1
```

### 5. Verify Package Contents

```bash
cargo package --list
```

### 6. Dry Run

```bash
cargo publish --dry-run
```

### 7. Publish

```bash
cargo publish
```

## Post-Publishing

1. Verify the package on [https://crates.io/crates/tuiw](https://crates.io/crates/tuiw)
2. Update documentation on docs.rs
3. Announce the release

## Troubleshooting

**Error: "files in working directory contain changes"**
- Commit all changes before publishing
- Or use `--allow-dirty` flag (not recommended)

**Error: "crate name is already taken"**
- The name `tuiw` must be available on crates.io
- Check ownership at https://crates.io/crates/tuiw

**Error: "failed to verify package"**
- Ensure all tests pass: `cargo test`
- Fix any compilation errors

## Versioning

Follow [Semantic Versioning](https://semver.org/):
- MAJOR version for incompatible API changes
- MINOR version for backwards-compatible functionality
- PATCH version for backwards-compatible bug fixes
