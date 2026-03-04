set shell := ["flox", "activate", "--", "sh", "-cu"]

[private]
default:
    @just --list --justfile {{ justfile() }}

# Run a subset of checks as pre-commit hooks
pre-commit-inner:
    #!/usr/bin/env -S parallel --shebang --ungroup --jobs {{ num_cpus() }}
    just prettier true
    just format-toml true
    just format-rust true
    just lint-github-actions
    just lint-markdown
    just lint-yaml
    just test-rust

pre-commit:
    just pre-commit-inner

# Check documentation
check-docs:
    cargo doc --all-features --no-deps

# Check features with cargo-hack
check-features:
    cargo hack --feature-powerset check --lib --tests

# Check latest dependencies
check-deps-latest:
    cargo update
    RUSTFLAGS="-D deprecated" cargo check --all-features --all-targets

# Check minimal dependencies (requires nightly)
check-deps-minimal:
    cargo +nightly update -Z direct-minimal-versions
    cargo check --all-features --all-targets

# Format JSON files
format-json fix="false": (prettier fix "{json,json5}")

# Format Markdown files
format-markdown fix="false": (prettier fix "md")

# Format Rust files
format-rust fix="false":
    cargo fmt {{ if fix != "true" { "--check" } else { "" } }}

# Format Just files
format-just fix="false":
    just --fmt {{ if fix != "true" { "--check" } else { "" } }} --unstable

# Format TOML files
format-toml fix="false":
    taplo fmt {{ if fix != "true" { "--diff" } else { "" } }}

# Format YAML files
format-yaml fix="false": (prettier fix "{yaml,yml}")

# Lint GitHub Actions workflows
lint-github-actions:
    zizmor -p .

# Lint Markdown files
lint-markdown:
    markdownlint **/*.md

# Lint Rust files
lint-rust:
    cargo clippy --all-targets --all-features -- -D warnings

# Lint TOML files
lint-toml:
    taplo check

# Lint YAML files
lint-yaml:
    yamllint .

# Auto-format files with prettier
prettier fix="false" extension="*":
    prettier {{ if fix == "true" { "--write" } else { "--list-different" } }} --ignore-unknown "**/*.{{ extension }}"

# Run the tests
test-rust:
    cargo test --all-features --all-targets

# Run tests with coverage
test-coverage:
    cargo tarpaulin \
        --all-features \
        --engine llvm \
        --exclude doco-derive \
        --out xml \
        --skip-clean \
        --timeout 120 \
        --verbose \
        --workspace

# Publish crates to crates.io
publish:
    cargo publish -p doco-derive -v --all-features
    cargo publish -p doco -v --all-features
