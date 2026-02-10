# Phoenix Tools - Quick Reference

## 🚀 Quick Start Commands

### Development
```bash
# Start full dev environment (recommended)
make dev

# CLI-only development (faster)
make dev-cli

# GUI-only development
make dev-gui

# Open shell in dev container
make shell
```

### Production
```bash
# Build production image
make build-prod

# Run production container
make prod

# Run CLI commands
docker-compose run --rm cli detect
docker-compose run --rm cli vault list
```

### Testing
```bash
# Run all tests
make test

# Run tests in watch mode
make test-watch

# Format and lint
make fmt && make lint
```

## 📦 Available Targets

| Target | Description | Size | Use Case |
|--------|-------------|------|----------|
| `dev` | Full development environment | ~3GB | Full-stack development |
| `dev-cli` | CLI-only development | ~3GB | Backend development |
| `dev-gui` | GUI-only development | ~3GB | Frontend development |
| `prod` | Production with GUI | ~800MB | Production deployment |
| `cli` | CLI-only production | ~200MB | Headless servers, CI/CD |

## 🔧 Development Workflow

### Inside Dev Container

```bash
# Hot reload CLI
cargo watch -x 'run --bin phoenix'

# Hot reload GUI
cd ui && npm run tauri dev

# Run specific commands
phoenix detect
phoenix forensics deep-scan --device /dev/ttyUSB0
phoenix vault create --device COM3 --name "backup"

# Run tests
cargo test --workspace
cargo test --package phoenix-lib --test integration_test
```

### Code Quality

```bash
# Format
cargo fmt --all

# Lint
cargo clippy --workspace --all-features -- -D warnings

# Check compilation
cargo check --workspace
```

## 🐛 Debugging

### Enable Verbose Logging

```bash
# Inside container
export RUST_LOG=trace,phoenix=trace
phoenix detect

# Or via docker-compose
docker-compose run --rm -e RUST_LOG=trace dev phoenix detect
```

### Attach to Running Container

```bash
# Start in background
docker-compose up -d dev

# Attach to it
docker attach phoenix-dev

# Or open new shell
docker-compose exec dev bash
```

## 🔌 Hardware Access

### Check USB Devices

```bash
# Inside container
lsusb
ls -la /dev/ttyUSB*
ls -la /dev/ttyACM*

# Test device detection
phoenix detect
```

### Supported Devices

- **Amlogic**: S905W, S905X, S912 (VID: 1b8e)
- **Rockchip**: RK3229, RK3399 (VID: 2207)
- **Allwinner**: H3, H6 (VID: 1f3a)
- **UART**: CH340 (VID: 1a86), CP2102 (VID: 10c4)

## 📊 Performance Tips

### Speed Up Rebuilds

1. **Use persistent volumes** (already configured)
2. **Enable BuildKit cache mounts** (already enabled)
3. **Don't rebuild unnecessarily**:
   ```bash
   # Only rebuild when Dockerfile changes
   make build-dev
   
   # Otherwise, just start containers
   make dev
   ```

### Clean Up Space

```bash
# Remove containers and volumes
make clean

# Remove everything including images (CAUTION)
make clean-all

# Prune Docker system
docker system prune -a
```

## 🐧 Platform-Specific Notes

### Linux

- **X11 forwarding**: Already configured
- **USB access**: Requires privileged mode (configured)
- **udev rules**: Automatically installed in containers

```bash
# Allow X11 connections (host)
xhost +local:docker

# Start dev environment
make dev
```

### Windows (WSL2)

- **USB passthrough**: Requires USB/IP forwarding
- **Display**: Use VcXsrv or Windows 11 WSLg

```powershell
# In WSL2
make dev

# Check USB devices
lsusb
```

### macOS

- **USB access**: Limited support (use native installation)
- **Display**: Use XQuartz for X11 forwarding

## 🛠 Troubleshooting

### "Cannot connect to Docker daemon"

```bash
# Start Docker Desktop
# Or on Linux:
sudo systemctl start docker
```

### "Permission denied" on /dev/ttyUSB0

```bash
# Host machine (Linux):
sudo usermod -a -G dialout $USER
# Log out and back in

# Or run with privileged mode (already set)
```

### "Port already in use"

```bash
# Find and kill process on port
# Linux:
sudo lsof -ti:1420 | xargs kill -9

# Windows:
netstat -ano | findstr :1420
taskkill /PID <PID> /F
```

### "Out of space"

```bash
# Check disk usage
docker system df

# Clean up
make clean
docker builder prune -a
```

## 📝 Common Tasks

### Create Firmware Backup

```bash
docker-compose run --rm cli vault create \
  --device /dev/ttyUSB0 \
  --name "my-box-backup-$(date +%Y%m%d)"
```

### Scan for Malware

```bash
docker-compose run --rm cli security scan \
  --image /path/to/firmware.img \
  --format json > scan-results.json
```

### Generate Remote Config

```bash
docker-compose run --rm cli remote generate-conf \
  --name "X96" \
  --output /app/configs/remote.conf
```

### Run Forensics

```bash
docker-compose run --rm cli forensics deep-scan \
  --device /dev/ttyUSB0 \
  --output forensics-report.json
```

## 🎯 Next Steps

1. **Read full documentation**: `docs/DOCKER.md`
2. **Check project README**: `README.md`
3. **Explore configs**: `configs/` directory
4. **Try examples**: Run `phoenix --help` for all commands

## 💡 Pro Tips

- Use `make help` to see all available commands
- Keep dev environment running in background with `make up`
- Use `make logs` to follow container logs
- Commit from inside container to get "Assisted-By: cagent" trailer
- Check `docker-compose.yml` for environment customization
