.PHONY: build release debug clean dev dev-stop lint format test

PREFIX ?= /usr/local
EDITOR_PORT ?= 9090
MARKSERV_PORT ?= 9091

build: release

release:
	swift build -c release

debug:
	swift build

clean:
	swift package clean
	rm -rf .build

test:
	swift test

lint:
	@if command -v swiftlint >/dev/null 2>&1; then \
		swiftlint lint --strict; \
	else \
		echo "SwiftLint not installed. Install with: brew install swiftlint"; \
		exit 1; \
	fi

format:
	@if command -v swiftformat >/dev/null 2>&1; then \
		swiftformat .; \
	else \
		echo "SwiftFormat not installed. Install with: brew install swiftformat"; \
		exit 1; \
	fi

format-check:
	@if command -v swiftformat >/dev/null 2>&1; then \
		swiftformat --lint .; \
	else \
		echo "SwiftFormat not installed. Install with: brew install swiftformat"; \
		exit 1; \
	fi

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
