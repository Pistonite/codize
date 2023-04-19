prep:
    cargo fmt
    cargo clippy --all-targets --all-features -- -D warnings
    cargo doc
    cargo package