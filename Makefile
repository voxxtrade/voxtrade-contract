all: lint build test

build:
	@echo "Building optimized WASM binaries..."
	cargo build --target wasm32-unknown-unknown --release

test:
	@echo "Running tests..."
	cargo test

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

lint: fmt clippy

quickstart:
	docker-compose up -d

bindings: build
	bash scripts/deploy.sh

clean:
	cargo clean

