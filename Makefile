.PHONY: build run clean schemas docker-up docker-down

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
