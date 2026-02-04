.PHONY: build release debug clean dev dev-stop lint format test install

PREFIX ?= /usr/local
EDITOR_PORT ?= 9090
MARKSERV_PORT ?= 9091

build: release
	@cp target/release/md2cb ./md2cb

release:
	cargo build --release

debug:
	cargo build

clean:
	cargo clean
	rm -f ./md2cb

test:
	cargo test

lint:
	@if command -v cargo-clippy >/dev/null 2>&1; then \
		cargo clippy -- -D warnings; \
	else \
		echo "Clippy not installed. Install with: rustup component add clippy"; \
		exit 1; \
	fi

format:
	cargo fmt

format-check:
	cargo fmt -- --check

install: release
	install -d $(PREFIX)/bin
	install target/release/md2cb $(PREFIX)/bin/

dev: dev-stop
	@echo "Starting dev servers..."
	@docker run -d --rm --name md2cb-editor -p $(EDITOR_PORT):80 -v $(PWD)/test:/usr/share/caddy:ro caddy:latest >/dev/null
	@pnpx markserv -p $(MARKSERV_PORT) ./test/demo.md >/dev/null 2>&1 &
	@sleep 1
	@echo "Rich text editor: http://localhost:$(EDITOR_PORT)"
	@echo "Markdown preview: http://localhost:$(MARKSERV_PORT)/demo.md"
	@open http://localhost:$(EDITOR_PORT)
	@open http://localhost:$(MARKSERV_PORT)/demo.md
	@echo ""
	@echo "Run 'make dev-stop' to stop servers"

dev-stop:
	@-docker stop md2cb-editor >/dev/null 2>&1 || true
	@-pkill -f "markserv.*$(MARKSERV_PORT)" 2>/dev/null || true
