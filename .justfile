default: check test fmt

check:
    cargo check

test:
    cargo test --all -- --nocapture

fmt:
    cargo fmt --all -- --check

build-windows:
    cargo build --target x86_64-pc-windows-gnu --release
