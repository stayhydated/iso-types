default:
    @just --list

# Regenerate ICU snapshots, selected catalogs, locale scope, and Fluent registration.
generate:
    python3 -m tools.codegen

# Verify the generated files without writing.
generate-check:
    python3 -m tools.codegen --check

# Exercise data-selection, literal-escaping, and output-checking contracts.
codegen-test:
    python3 -m unittest discover -s tools/codegen/tests -v

# Format handwritten and generated Rust.
format:
    cargo fmt --all

format-check:
    cargo fmt --all --check

# Test default, minimal, and complete catalog/Fluent configurations.
test:
    cargo test --workspace --locked
    cargo test -p iso-types --no-default-features --locked
    cargo test --workspace --all-features --locked

clippy:
    cargo clippy --workspace --all-features --all-targets --locked -- -D warnings

# Test each optional regional feature without default-feature unification.
test-features:
    python3 tools/check_features.py

docs:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --locked

# Package and verify both public crates without uploading.
package:
    cargo publish --workspace --dry-run --allow-dirty --locked

# Run the complete repository verification sequence.
verify: codegen-test generate-check format-check test test-features clippy docs

# Assemble the book, Markdown exports, and Dioxus examples for GitHub Pages.
web-build:
    cargo xtask build book
    cargo xtask build llms-txt
    cargo xtask build web

# Develop the Dioxus examples with live reload.
web: web-build
    dx serve --package web

# Preview the assembled Pages artifact, including direct route navigation.
web-preview: web-build
    cargo xtask preview web

# Validate the web package's typed messages and both UI languages.
web-i18n:
    cargo es-fluent doctor --package web
    cargo es-fluent check --package web --all-locales
