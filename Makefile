# Weave — portable task runner.
#
# Every target works from a checkout on Linux/macOS/Windows with only
# cargo installed. Nix targets are optional and require flakes enabled.

CARGO ?= cargo
CARGO_FLAGS ?=

.PHONY: help build release test lint fmt fmt-check check doc demo playground panel clean nix-dev nix-check nix-build js-test js-fixture wasm

help: ## Show this help
	@grep -hE '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-14s\033[0m %s\n", $$1, $$2}'

build: ## Build the portable core and the terminal demos
	$(CARGO) build --workspace $(CARGO_FLAGS)

release: ## Build everything in release mode
	$(CARGO) build --workspace --release $(CARGO_FLAGS)

test: ## Run the whole test suite
	$(CARGO) test --workspace $(CARGO_FLAGS)

lint: ## Run clippy with warnings denied
	$(CARGO) clippy --workspace --all-targets -- -D warnings

fmt: ## Format the workspace
	$(CARGO) fmt --all

fmt-check: ## Check formatting
	$(CARGO) fmt --all -- --check

check: fmt-check lint test ## Everything CI runs

doc: ## Build the API documentation
	$(CARGO) doc --workspace --no-deps

demo: ## Run the pill-to-panel demo
	$(CARGO) run -p button-to-panel

playground: ## Run the spring playground
	$(CARGO) run -p spring-playground

js-test: ## Test weave.js against the Rust solver (needs node)
	node --test js/test

js-fixture: ## Regenerate the Rust reference samples used by js-test
	$(CARGO) run -q -p weave-core --example js_fixture > js/test/fixtures/springs.json

wasm: ## Build the optional wasm engine into js/pkg (needs wasm-bindgen-cli)
	$(CARGO) build -p weave-web --features web --target wasm32-unknown-unknown --release
	wasm-bindgen --target web --out-dir js/pkg target/wasm32-unknown-unknown/release/weave_web.wasm

clean: ## Remove build artifacts
	$(CARGO) clean

nix-dev: ## Enter the Nix development shell
	nix develop

nix-check: ## Build and test inside the Nix sandbox
	nix flake check

nix-build: ## Build the Nix packages
	nix build
