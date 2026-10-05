.PHONY: all help install build release clippy rustfmt ruff mypy linting test clean

all: install

help: ## Show this help menu (list all available commands)
	@grep -E '^[a-zA-Z0-9_]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'

install: ## Install all development dependencies and sync the virtual environment
	@uv sync --locked --only-dev --no-install-local
	@uv run maturin develop --uv

build: ## Build the source distribution for the Python package
	@uv build --sdist

release: ## Publish the current build to PyPI
	@uv publish

clippy: ## Run `clippy` to check for Rust-specific linting errors and best practices
	@cargo clippy

rustfmt: ## Check that all Rust code adheres to standard formatting rules
	@cargo fmt --check

ruff: ## Run `ruff` to perform both linting and formatting verification
	@uv run ruff check --diff
	@uv run ruff format --diff

mypy: ## Perform static type checking on the Python source code using `mypy`
	@uv run mypy python

linting: clippy rustfmt ruff mypy ## Run all linting and formatting checks for both Rust and Python

test: ## Execute the full suite of Rust unit tests
	@cargo test --lib

clean: ## Remove Python byte-code, build artifacts, and clear the Cargo build cache
	@find python -type f \( -iname '*.py[codz]' -or -iname '*.so' \) -delete
	@find python -type d -iname '__pycache__' -delete
	@cargo clean
