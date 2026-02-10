.PHONY: help build dev prod test clean up down logs shell

# Default target
.DEFAULT_GOAL := help

# Colors for output
BLUE := \033[0;34m
GREEN := \033[0;32m
YELLOW := \033[0;33m
RED := \033[0;31m
NC := \033[0m # No Color

## help: Display this help message
help:
	@echo "$(BLUE)Phoenix Tools - Docker Commands$(NC)"
	@echo ""
	@echo "$(GREEN)Development:$(NC)"
	@echo "  make dev          - Start full development environment"
	@echo "  make dev-cli      - Start CLI-only development"
	@echo "  make dev-gui      - Start GUI-only development"
	@echo "  make shell        - Open bash shell in dev container"
	@echo "  make logs         - Follow logs from dev container"
	@echo ""
	@echo "$(GREEN)Building:$(NC)"
	@echo "  make build        - Build all images"
	@echo "  make build-dev    - Build development image only"
	@echo "  make build-prod   - Build production image only"
	@echo "  make build-cli    - Build CLI-only image"
	@echo ""
	@echo "$(GREEN)Production:$(NC)"
	@echo "  make prod         - Run production container"
	@echo "  make cli          - Run CLI container"
	@echo ""
	@echo "$(GREEN)Testing:$(NC)"
	@echo "  make test         - Run all tests in container"
	@echo "  make test-watch   - Run tests in watch mode"
	@echo ""
	@echo "$(GREEN)Maintenance:$(NC)"
	@echo "  make clean        - Remove containers and volumes"
	@echo "  make clean-all    - Remove everything including images"
	@echo "  make up           - Start containers in background"
	@echo "  make down         - Stop all containers"
	@echo "  make restart      - Restart all containers"
	@echo ""

## dev: Start full development environment
dev:
	@echo "$(GREEN)Starting development environment...$(NC)"
	docker-compose up dev

## dev-cli: Start CLI-only development
dev-cli:
	@echo "$(GREEN)Starting CLI development...$(NC)"
	docker-compose up dev-cli

## dev-gui: Start GUI-only development
dev-gui:
	@echo "$(GREEN)Starting GUI development...$(NC)"
	docker-compose up dev-gui

## shell: Open bash shell in dev container
shell:
	@echo "$(GREEN)Opening development shell...$(NC)"
	docker-compose exec dev bash || docker-compose run --rm dev bash

## logs: Follow logs from dev container
logs:
	docker-compose logs -f dev

## build: Build all images
build:
	@echo "$(GREEN)Building all images...$(NC)"
	docker-compose build

## build-dev: Build development image only
build-dev:
	@echo "$(GREEN)Building development image...$(NC)"
	docker-compose build dev

## build-prod: Build production image only
build-prod:
	@echo "$(GREEN)Building production image...$(NC)"
	docker-compose build prod

## build-cli: Build CLI-only image
build-cli:
	@echo "$(GREEN)Building CLI image...$(NC)"
	docker-compose build cli

## prod: Run production container
prod:
	@echo "$(GREEN)Starting production container...$(NC)"
	docker-compose up prod

## cli: Run CLI container
cli:
	@echo "$(GREEN)Running CLI container...$(NC)"
	docker-compose run --rm cli

## test: Run all tests in container
test:
	@echo "$(GREEN)Running tests...$(NC)"
	docker-compose run --rm test

## test-watch: Run tests in watch mode
test-watch:
	@echo "$(GREEN)Running tests in watch mode...$(NC)"
	docker-compose run --rm dev cargo watch -x test

## up: Start containers in background
up:
	@echo "$(GREEN)Starting containers...$(NC)"
	docker-compose up -d

## down: Stop all containers
down:
	@echo "$(YELLOW)Stopping containers...$(NC)"
	docker-compose down

## restart: Restart all containers
restart:
	@echo "$(YELLOW)Restarting containers...$(NC)"
	docker-compose restart

## clean: Remove containers and volumes
clean:
	@echo "$(YELLOW)Cleaning up containers and volumes...$(NC)"
	docker-compose down -v
	@echo "$(GREEN)Cleanup complete!$(NC)"

## clean-all: Remove everything including images
clean-all:
	@echo "$(RED)WARNING: This will remove all containers, volumes, and images!$(NC)"
	@read -p "Are you sure? (y/N): " confirm && [ "$$confirm" = "y" ] || exit 1
	docker-compose down -v --rmi all
	@echo "$(GREEN)Full cleanup complete!$(NC)"

## fmt: Format code in container
fmt:
	@echo "$(GREEN)Formatting code...$(NC)"
	docker-compose run --rm dev cargo fmt --all

## lint: Run clippy linter
lint:
	@echo "$(GREEN)Running clippy...$(NC)"
	docker-compose run --rm dev cargo clippy --workspace --all-features -- -D warnings

## check: Run cargo check
check:
	@echo "$(GREEN)Running cargo check...$(NC)"
	docker-compose run --rm dev cargo check --workspace --all-features

## outdated: Check for outdated dependencies
outdated:
	@echo "$(GREEN)Checking for outdated dependencies...$(NC)"
	docker-compose run --rm dev cargo outdated

## audit: Security audit
audit:
	@echo "$(GREEN)Running security audit...$(NC)"
	docker-compose run --rm dev cargo audit || echo "Install cargo-audit first: cargo install cargo-audit"
