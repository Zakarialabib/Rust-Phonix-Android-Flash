# Phoenix Tools

Rust-based CLI and GUI for liberating Android TV boxes.

## Quick Start

### Windows
```powershell
# Run the bootstrap script to set up everything (including GUI)
.\scripts\bootstrap.ps1 -Gui
```

### Linux/macOS
```bash
chmod +x scripts/bootstrap.sh
./scripts/bootstrap.sh --gui
```

## Running the GUI manually

If you prefer to run the GUI manually:

```bash
cd phoenix-tools/ui
npm install
npm run tauri dev
```

Alternatively, start the frontend dev server:

```bash
npm run dev
# Open http://localhost:5173
```

## Usage

### The Golden Rule: Vault Before Flash
Always create a secure backup of your original firmware before any flash operation. This preserves unique MAC addresses, WiFi calibration (NVRAM), and HDCP keys.

```bash
# Create encrypted backup of original firmware
phoenix vault create --device COM3 --name "tx3-original"

# List available backups
phoenix vault list

# Verify backup integrity
phoenix vault verify --name "tx3-original"

# Restore if something goes wrong
phoenix vault restore --name "tx3-original" --device COM3
```

### Hardware Forensics & Detection
```bash
# Detect connected devices (USB/UART)
phoenix detect

# Deep forensics (RAM vendor, PCB variant, WiFi chip)
phoenix forensics deep-scan --device COM3

# Check specific PCB variant (Critical for S905W p281 vs p282)
phoenix forensics pcb-variant --device COM3
```

### Build & Flash
```bash
# Check compatibility against target firmware (e.g., SlimBoxTV)
phoenix check --profile my-tx3.yaml --firmware slimbox_v15.img

# Build customized firmware
phoenix build --profile minimal --board my-tx3.yaml

# Flash to SD card or eMMC
phoenix flash --target sd --device /dev/sdb --image output/phoenix.img
```

## Project Structure

```
phoenix-tools/
├── phoenix-cli/      # CLI tool (Rust)
├── phoenix-lib/      # Shared library (Rust)
├── ui/               # Desktop GUI (Tauri 2.0)
│   ├── src/          # Frontend logic
│   ├── src-tauri/    # Rust backend
│   └── package.json
├── configs/          # Device YAML configs
├── recipes/          # Build scripts
└── scripts/          # Bootstrap scripts
```

## License

Apache-2.0 OR MIT

## New Commands (alpha)

- Forensics: aggregates USB and UART signals to infer device profile
- Check: evaluates hardware/firmware compatibility via a rule-based matrix
- Patch Plan: outputs a sequenced plan of DTB overlays and blob operations
- Validate: hardware-in-loop stubs for post-flash sanity checks
- **Security**: scan firmware for malware (Corejava botnet, BadBox, etc.)
- **Remote**: generate IR remote configurations (remote.conf, keylayout files)

### Security Scanning

```bash
# Scan extracted firmware for malware
phoenix security scan --image /path/to/extracted/firmware --format text

# JSON output for automation
phoenix security scan --image /path/to/firmware --format json
```

### Remote Configuration

```bash
# List available remote configurations
phoenix remote list

# Generate remote.conf for X96 Mini
phoenix remote generate-conf --name "X96" --output remote.conf

# Generate Android keylayout
phoenix remote generate-keylayout --name "X96" --output Generic.kl
```

## Platform Prerequisites

### Windows
- [Zadig](https://zadig.akeo.ie/) (for USB drivers)
- USB Burning Tool drivers (Amlogic) or Rockchip DriverAssistant
- **Critical**: CH340/CP2102 UART drivers (3.3V logic only)

### Linux
```bash
# Ubuntu/Debian
sudo apt install libusb-1.0-0-dev libudev-dev minicom

# For Rockchip
sudo apt install libusb-1.0-0 python3-pyusb
sudo cp 50-rockchip.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
```

### macOS
```bash
brew install libusb minicom
# Note: Some Amlogic tools require virtualized Linux due to driver restrictions
```

## Platform-Specific Entry

### Amlogic (S905W, S905X, S912)
- **Entry**: USB Burning Tool protocol (VID: 1B8E)
- **Unlock**: UART interrupt or eMMC pin short (pins 8-9)
- **Warning**: 3.3V UART only (5V will destroy SoC)

### Rockchip (RK3229, RK3399)
- **Entry**: Maskrom mode (RKDevTool)
- **Unlock**: Short CLK/D0 or use Recovery button
- **Baud**: 1.5Mbps (non-standard)

### Allwinner (H3, H6)
- **Entry**: FEL mode (USB-OTG)
- **Unlock**: Hold FEL button during boot

## Troubleshooting

### "Device not detected in Download mode"
- **USB Ports**: Try USB 2.0 ports instead of 3.0.
- **Cables**: Ensure you are using a high-quality USB-A to USB-A cable (not a standard phone charger cable).
- **Power**: Some boxes require separate 5V 2A power even when connected via USB.

### "Boot loop after flash"
- **RAM Timing**: Incorrect DDR timing detected? Run `phoenix forensics ram-vendor`.
- **DTB Mismatch**: Ensure you used the correct pcb-variant (p281 vs p282).

## Resources & Attribution
- [Armbian](https://www.armbian.com/) - Linux for ARM.

## Setup & Prerequisites

### Windows Users
*   **Rust**: Install via [rustup.rs](https://rustup.rs/) (default installation).
*   **Node.js**: Install Node.js 18+ (LTS) via [nodejs.org](https://nodejs.org/).
*   **Visual Studio Build Tools**: "Desktop development with C++" workload required for compiling system APIs.

### Setup Steps
1.  **Clone the Repository**
    ```powershell
    git clone https://github.com/phoenix-arm/phoenix-tools.git
    cd phoenix-tools
    ```

2.  **Frontend Setup**
    ```powershell
    cd ui
    npm install
    ```

3.  **Run the Application**
    ```powershell
    # From 'ui' directory
    npm run tauri dev
    ```

## Common Issues

### "WinRAR" or "Unzip" Loops
*   **Do NOT run older automated scripts** like `rockchip_automate.ps1`.
*   The application has **native Rust support** for archives. Do not use external shell commands.

### Build Errors
*   **"Linker not found"**: Ensure Visual Studio C++ Build Tools are installed.
*   **"WSL not found"**: Only required for compiling new firmware images, not for flashing.
