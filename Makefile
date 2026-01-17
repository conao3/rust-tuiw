all:

.PHONY: build
build:
	cargo build

.PHONY: release
release:
	cargo build --release

.PHONY: run
run:
	cargo run

.PHONY: test
test:
	cargo test

.PHONY: check
check:
	cargo check

.PHONY: clippy
clippy:
	cargo clippy

.PHONY: fmt
fmt:
	cargo fmt

.PHONY: clean
clean:
	cargo clean

.PHONY: watch
watch:
	cargo watch -x run

.PHONY: treefmt
treefmt:
	nix fmt

.PHONY: ci
ci:
	cargo fmt --check
	cargo clippy --all-targets --all-features -- -D warnings
	cargo test --all-features

.PHONY: integration
integration:
	cargo test --test '*' -- --ignored --test-threads=1

.PHONY: coverage
coverage:
	cargo llvm-cov --all-features --lcov --output-path lcov.info

.PHONY: release-check
release-check:
	cargo publish --dry-run
