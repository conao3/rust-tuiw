# Publishing to crates.io

The `Release` workflow (`.github/workflows/release.yml`) publishes `tuiw` to crates.io and creates the GitHub Release. Run it from the default branch:

```bash
gh workflow run release.yml --ref master -f version_bump=patch
```

`version_bump` is `patch`, `minor`, or `major`. One run does the following:

1. `check` bumps the version in `Cargo.toml` and `Cargo.lock` in a local commit, then runs `cargo publish --dry-run`.
2. `tag` pushes the version commit to `master` together with the annotated tag `v<version>`.
3. `build-binaries` builds `tuiw` for Linux (x86_64, x86_64 musl, aarch64) and macOS (x86_64, aarch64) from the tag.
4. `publish` publishes the crate with crates.io trusted publishing (OIDC). crates.io trusts `conao3/rust-tuiw` and `release.yml`, and the crate accepts trusted publishing only.
5. `github-release` creates the GitHub Release with generated notes and attaches the `tuiw-<target>.tar.gz` archives.

Keep the version in `Cargo.toml` on `master` at the latest published version, and do not push tags by hand. If a job after `tag` fails, rerun the failed jobs of the same run.

After the run, `git pull` the version commit and check the release:

```bash
gh release view v<version>
curl -s -A 'release-check/1.0' https://crates.io/api/v1/crates/tuiw | jq '.versions[0] | {num, trustpub_data}'
```

## Versioning

Follow [Semantic Versioning](https://semver.org/):
- MAJOR version for incompatible API changes
- MINOR version for backwards-compatible functionality
- PATCH version for backwards-compatible bug fixes
