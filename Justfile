prep:
    cargo fmt
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    cargo doc
    cargo package