
[group: "check"]
default: check

[group: "check"]
check: check-code check-fmt check-lint check-udeps check-tests

[group: "check"]
check-code:
    cargo check --workspace

[group: "check"]
check-tests:
    cargo test --workspace

[group: "check"]
check-fmt:
    cargo +nightly fmt --all -- --check

[group: "check"]
check-udeps:
    cargo +nightly udeps --workspace --locked

[group: "check"]
check-lint:
    cargo clippy

[group: "fix"]
fix: fix-fmt fix-lint

[group: "fix"]
fix-code:
    cargo fix --workspace

[group: "fix"]
fix-fmt:
    cargo +nightly fmt

[group: "fix"]
fix-lint:
    cargo clippy --fix --allow-dirty

windows_target := "x86_64-pc-windows-msvc"


[group: "build"]
build:
    cargo build --release --package cascade-app

[group: "build"]
[unix]
build-windows:
    cross build --release --target {{windows_target}} --package cascade-app

[group: "build"]
[windows]
build-windows: build

[group: "run"]
run:
    cargo run --package cascade-app

[group: "run"]
run-cli:
    cargo run --package cascade-cli
