build:
    cargo build --workspace --bin chip8-rs --release

test:
    cargo test --workspace

check:
    cargo check --workspace

clean:
    cargo clean
