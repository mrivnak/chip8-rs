build:
    cargo build --workspace --bin chip8-rs --release

build-wasm:
    cd chip8-rs && wasm-pack build --target web --release --out-dir ../target/pkg

test:
    cargo test --workspace

check:
    cargo check --workspace

clean:
    cargo clean
