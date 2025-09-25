.PHONY: fmt lint test build run-examples

fmt:
	cargo fmt

lint:
	cargo clippy -- -D warnings

test:
	cargo test

build:
	cargo build --release

run-examples:
	cargo run --bin autorisation -- --input examples/autorisation.yml --out autorisation_sortie.pdf
