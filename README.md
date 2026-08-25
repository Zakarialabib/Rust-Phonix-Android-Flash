# Rust Phoenix Android Tools 

![Version](https://img.shields.io/badge/version-0.1.0--alpha-blue.svg?style=for-the-badge)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green.svg?style=for-the-badge)
![Platform](https://img.shields.io/badge/platform-Windows%20|%20Linux%20|%20macOS-lightgrey.svg?style=for-the-badge)
![Stars](https://img.shields.io/github/stars/zakarialabib/phoenix-tools?style=for-the-badge)

> **"Hardware liberation should be as easy as installing Ubuntu on a laptop."**

**Rust Phoenix Android Tools** is an open-source circular computing platform that transforms e-waste (abandoned Android TV boxes) into secure, sovereign infrastructure. It is not just a flashing tool; it is a complete ecosystem where heterogenous ARM hardware becomes a unified compute fabric.

---

## 🚀 The Vision

50+ million Android TV boxes become obsolete annually. They are perfectly good computers (Quad-core ARM64, 2GB+ RAM) that are artificially limited.

**Phoenix** treats this e-waste as a resource:
1.  **Detect** unknown hardware via USB/UART forensics.
2.  **Unlock** bootloaders safely.
3.  **Repurpose** them into servers, nodes, or privacy tools.

---

## 🏗 System Architecture: The Three Layers

Phoenix is built on a "Three-Layer Cake" architecture:

### 1. The Hardware Liberation Engine (HLE)
*Safely turns a "brick" into a "blank canvas".*
- **SoC Sherlock**: Computer vision & forensic analysis to identify hardware.
- **BootROM Negotiator**: Handles low-level protocols (Amlogic, Rockchip, Allwinner).
- **The Vault**: **Mandatory** encrypted backup of original firmware (MAC, WiFi keys, DRM).

### 2. The Foundry (Image Build Pipeline)
*Transforms blank hardware into purpose-specific infrastructure.*
- **Intent-Based Builds**: You declare "I want a home server", Phoenix builds the OS.
- **Compatibility Matrix**: A rule engine that prevents invalid combinations (e.g., mismatched RAM timings).
- **Patch Engine**: Applies binary patches and DTB overlays automatically.

### 3. The Colony (Mesh Network)
*Connects liberated nodes.*
- **Roost**: Discovery and onboarding.
- **Molting**: OTA update system.

---

## 🛠 Supported Hardware

We maintain a strict **Compatibility Matrix** to ensure safety.

| SoC | Aliases | Architecture | Notes |
|-----|---------|--------------|-------|
| **S905W** | `amlogic_s905w` | Amlogic | Common in X96 Mini, TX3 Mini (P281/P282) |
| **S905X** | `amlogic_s905x` | Amlogic | HDR support, slightly different pinout |
| **H3** | `allwinner_h3` | Allwinner | Quad-core Cortex-A7 (Orange Pi compatible) |
| **RK3229** | `rk3229` | Rockchip | Legacy budget boxes |
| **RK3328** | `rk3328` | Rockchip | USB 3.0 support |
| **RK3399** | `rk3399` | Rockchip | High performance, big.LITTLE |

*Deep forensics can also detect RAM vendors (Samsung, Hynix, Micron) and WiFi chips (AP6212, RTL8189FS).*

---

## ⚡ Quick Start

### Windows
Run the bootstrap script to set up Rust, Node.js, and dependencies:
```powershell
.\scripts\bootstrap.ps1 -Gui
```

### Running Manually
```bash
cd ui
npm install
npm run tauri dev
```

---

## 🔮 The Phoenix Workflow: "The Five Rites"

1.  **Reconnaissance** (`phoenix forensics`): Deep scan of RAM vendor, PCB revision, and eMMC health.
2.  **Reservation** (`phoenix vault`): **Critical step.** Full eMMC dump and extraction of unique device keys.
3.  **Restoration** (`phoenix check` → `phoenix patch`): Validate hardware against the Compatibility Matrix.
4.  **Rebirth** (`phoenix flash`): Atomic flashing operation using SoC-specific protocols.
5.  **Reclamation** (`phoenix validate`): Hardware-in-loop testing before deployment.

---

## 🗺 Roadmap

| Phase | Goal | Features |
|-------|------|----------|
| **Now** | **Alpha (v0.1.0)** | Amlogic/Rockchip/Allwinner flashing, Device Detection, Tauri GUI. |
| **Q1 2026** | **Universal Support** | Text-based config profiles, full Rockchip/Allwinner support. |
| **Q2 2026** | **Modding Tools** | Image unpack/repack, "Phoenix Forge" for custom ROMs. |
| **Q3 2026** | **Security** | "Phoenix Doctor" malware scanner, RAM verification. |

---

## 👨‍💻 Creator & Contributors

**Creator & Lead Developer**  
**Zakaria Labib** ([@zakarialabib](https://github.com/zakarialabib))


We welcome contributions! Please see our [Contribution Guide](docs/guides/Community%20Contribution%20Guide.md).

---

## 📄 License

Distributed under the **MIT** or **Apache-2.0** license. See `LICENSE` for more information.
