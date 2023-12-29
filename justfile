build:
    cargo build --workspace --bin chip8-rs --release

build-wasm:
    cd chip8-rs && wasm-pack build --target web --release --out-dir ../target/pkg

install-micro-target:
    rustup target add armv7a-none-eabi

build-micro:
    cargo build --workspace --bin chip8-rs-micro --release --target armv7a-none-eabi

build-all: build build-micro

test:
    cargo test --workspace --exclude chip8-rs-micro

check:
    cargo check --workspace --exclude chip8-rs-micro
    cargo check --workspace --bin chip8-rs-micro --target armv7a-none-eabi

clean:
    cargo clean
