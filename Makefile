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
