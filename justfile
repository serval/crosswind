# List available recipes
help:
    just -l

# Run crosswind with manual configuration
@run:
    cargo run

# Run crosswind against your tailnet
@tailscale *ARGS:
    cargo run --bin tailscale {{ARGS}}

# Run the same checks we run in CI
@ci: test
    cargo clippy
    cargo fmt --check
    cargo deny check licenses

# Get security advisories from cargo-deny
@security:
    cargo deny check advisories

# Run tests with nextest
@test:
    cargo nextest run --all-targets

# Lint and automatically fix what we can fix
@fix:
    cargo clippy --fix --allow-dirty --allow-staged
    cargo fmt

# Install required linting/testing tools via cargo.
@install-tools:
    cargo install cargo-nextest
    cargo install cargo-deny

# Check for unused dependencies.
check-unused:
    cargo +nightly udeps --all
