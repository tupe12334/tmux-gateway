.PHONY: build run clean schemas docker-up docker-down test lint check

build: schemas

schemas:
	cargo build
	cargo run --bin export_schemas

run:
	cargo run --bin tmux-gateway

clean:
	cargo clean
	rm -f schemas/openapi.json schemas/schema.graphql

docker-up:
	docker compose up --build -d

docker-down:
	docker compose down

test:
	cargo test --workspace

lint:
	cargo fmt --all -- --check
	cargo clippy --workspace -- -D warnings

check: lint test
	cargo doc --workspace --no-deps
	cargo deny check
	cargo audit
	cargo machete
	cspell --no-progress "**"
