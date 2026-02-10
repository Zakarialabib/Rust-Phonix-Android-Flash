#!/usr/bin/env bash
# Phoenix Tools - Docker Helper for Linux/macOS
# Usage: ./docker-helper.sh <command>

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

function show_help() {
    echo -e "${CYAN}🐳 Phoenix Tools - Docker Commands${NC}"
    echo "=================================="
    echo ""
    echo -e "${GREEN}Development:${NC}"
    echo "  ./docker-helper.sh dev          - Start full development environment"
    echo "  ./docker-helper.sh dev-cli      - Start CLI-only development"
    echo "  ./docker-helper.sh dev-gui      - Start GUI-only development"
    echo "  ./docker-helper.sh shell        - Open bash shell in dev container"
    echo "  ./docker-helper.sh logs         - Follow container logs"
    echo ""
    echo -e "${GREEN}Building:${NC}"
    echo "  ./docker-helper.sh build        - Build all images"
    echo "  ./docker-helper.sh build-dev    - Build development image"
    echo "  ./docker-helper.sh build-prod   - Build production image"
    echo ""
    echo -e "${GREEN}Production:${NC}"
    echo "  ./docker-helper.sh prod         - Run production container"
    echo "  ./docker-helper.sh cli <args>   - Run CLI command"
    echo ""
    echo -e "${GREEN}Testing:${NC}"
    echo "  ./docker-helper.sh test         - Run all tests"
    echo "  ./docker-helper.sh fmt          - Format code"
    echo "  ./docker-helper.sh lint         - Run linter"
    echo ""
    echo -e "${GREEN}Maintenance:${NC}"
    echo "  ./docker-helper.sh clean        - Remove containers and volumes"
    echo "  ./docker-helper.sh clean-all    - Remove everything (CAUTION)"
    echo "  ./docker-helper.sh up           - Start containers in background"
    echo "  ./docker-helper.sh down         - Stop all containers"
    echo "  ./docker-helper.sh restart      - Restart containers"
    echo ""
    echo -e "${YELLOW}Examples:${NC}"
    echo "  ./docker-helper.sh cli detect"
    echo "  ./docker-helper.sh cli vault list"
    echo "  ./docker-helper.sh cli forensics deep-scan --device /dev/ttyUSB0"
    echo ""
}

function check_docker() {
    if ! docker ps &> /dev/null; then
        echo -e "${RED}❌ Docker is not running. Please start Docker.${NC}"
        exit 1
    fi
}

# Main command router
COMMAND="${1:-help}"
shift || true

check_docker

case "$COMMAND" in
    help)
        show_help
        ;;
    
    dev)
        echo -e "${GREEN}🚀 Starting development environment...${NC}"
        docker-compose up dev
        ;;
    
    dev-cli)
        echo -e "${GREEN}🚀 Starting CLI development...${NC}"
        docker-compose up dev-cli
        ;;
    
    dev-gui)
        echo -e "${GREEN}🚀 Starting GUI development...${NC}"
        # Allow X11 connections
        xhost +local:docker 2>/dev/null || true
        docker-compose up dev-gui
        ;;
    
    shell)
        echo -e "${GREEN}🐚 Opening development shell...${NC}"
        docker-compose exec dev bash || docker-compose run --rm dev bash
        ;;
    
    logs)
        docker-compose logs -f dev
        ;;
    
    build)
        echo -e "${GREEN}🔨 Building all images...${NC}"
        docker-compose build
        ;;
    
    build-dev)
        echo -e "${GREEN}🔨 Building development image...${NC}"
        docker-compose build dev
        ;;
    
    build-prod)
        echo -e "${GREEN}🔨 Building production image...${NC}"
        docker-compose build prod
        ;;
    
    prod)
        echo -e "${GREEN}🚀 Starting production container...${NC}"
        docker-compose up prod
        ;;
    
    cli)
        if [ $# -eq 0 ]; then
            echo -e "${RED}❌ Please specify CLI arguments. Example: cli detect${NC}"
            exit 1
        fi
        echo -e "${GREEN}🔧 Running CLI command: $@${NC}"
        docker-compose run --rm cli "$@"
        ;;
    
    test)
        echo -e "${GREEN}🧪 Running tests...${NC}"
        docker-compose run --rm test
        ;;
    
    fmt)
        echo -e "${GREEN}✨ Formatting code...${NC}"
        docker-compose run --rm dev cargo fmt --all
        ;;
    
    lint)
        echo -e "${GREEN}🔍 Running clippy...${NC}"
        docker-compose run --rm dev cargo clippy --workspace --all-features -- -D warnings
        ;;
    
    up)
        echo -e "${GREEN}▶️  Starting containers...${NC}"
        docker-compose up -d
        ;;
    
    down)
        echo -e "${YELLOW}⏹️  Stopping containers...${NC}"
        docker-compose down
        ;;
    
    restart)
        echo -e "${YELLOW}🔄 Restarting containers...${NC}"
        docker-compose restart
        ;;
    
    clean)
        echo -e "${YELLOW}🧹 Cleaning up containers and volumes...${NC}"
        read -p "Are you sure? (y/N): " confirm
        if [ "$confirm" = "y" ] || [ "$confirm" = "Y" ]; then
            docker-compose down -v
            echo -e "${GREEN}✅ Cleanup complete!${NC}"
        else
            echo -e "${RED}❌ Cancelled${NC}"
        fi
        ;;
    
    clean-all)
        echo -e "${RED}⚠️  WARNING: This will remove all containers, volumes, and images!${NC}"
        read -p "Are you ABSOLUTELY sure? (y/N): " confirm
        if [ "$confirm" = "y" ] || [ "$confirm" = "Y" ]; then
            docker-compose down -v --rmi all
            echo -e "${GREEN}✅ Full cleanup complete!${NC}"
        else
            echo -e "${RED}❌ Cancelled${NC}"
        fi
        ;;
    
    *)
        echo -e "${RED}❌ Unknown command: $COMMAND${NC}"
        echo ""
        show_help
        exit 1
        ;;
esac
