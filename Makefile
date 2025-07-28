.PHONY: clean-build docs lint build check-web build-web hot run run-web

.ONESHELL: # Use one shell per target
	SHELL := /bin/bash
	# Stop excecution on any error
	.SHELLFLAGS = -ec

clean-build:
	cargo clean && make build-dev && make lint && make check-web

docs:
	cargo doc --open --no-deps --workspace

lint:
	cargo clippy -- -D warnings
	cargo fmt --all -- --check
	cargo machete

build-dev:
	cargo build

build:
	cargo build --release

check-web:
	cargo check --profile ci --no-default-features --features web --target wasm32-unknown-unknown

build-web:
	cargo binstall --locked -y --force wasm-bindgen-cli
	cargo binstall --locked -y --force wasm-opt
	bevy build --locked --release --features=web --yes web --bundle

hot:
	BEVY_ASSET_ROOT="." dx serve --hot-patch

run:
	cargo run

run-web:
	bevy run web --headers="Cross-Origin-Opener-Policy:same-origin" --headers="Cross-Origin-Embedder-Policy:credentialless"
