all: lint build test

build:
	@echo "Building optimized WASM binaries for Soroban..."
	cargo build --target wasm32-unknown-unknown --release

test:
	@echo "Running all smart contract tests..."
	cargo test

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

lint: fmt clippy

clean:
	cargo clean

