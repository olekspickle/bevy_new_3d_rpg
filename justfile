clean:
    cargo clean

build:
    cargo build

fmt:
    cargo fmt --all -- --check

doc:
    cargo doc --locked --profile ci --document-private-items --no-deps

clippy:
    cargo clippy --all-targets -- -D warnings

lint: clippy fmt doc check-web

clean-build: clean build lint

check-web:
    cargo check \
        --config 'profile.web.inherits="dev"' \
        --profile ci \
        --no-default-features \
        --features web \
        --target wasm32-unknown-unknown

install-wasm-deps:
    cargo binstall --locked -y --force wasm-bindgen-cli wasm-opt

build-web:
    bevy build --locked --release --no-default-features --features=web --yes web --bundle

web:
    bevy run --release --no-default-features --features web web -U multi-threading

web-dev:
    bevy run --no-default-features --features web,dev web -U multi-threading

hot:
    dx serve --hot-patch --features dev_native

tracy:
    cargo run --features debug --release
