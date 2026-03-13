.PHONY: build run clean schemas

build: schemas

schemas:
	cargo build
	cargo run --bin export_schemas

run:
	cargo run --bin tmux-gateway

clean:
	cargo clean
	rm -f schemas/openapi.json schemas/schema.graphql
