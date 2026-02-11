# Phoenix-ARM Developer Guidelines

This document outlines the core principles, mental models, and architectural decisions for the Phoenix-ARM project. New developers (and agents) should internalize these concepts before contributing.

## The Mental Model: "A Compiler, Not A Script"

We are not building a loose collection of shell scripts. We are building a **compiler for hardware liberation**.

A compiler takes high-level intent ("make this box a signage node") and lowers it through rigorous abstraction layers:
- **Intent** → **OS Profile** → **Kernel Config** → **DTB Selection** → **Binary Image** → **Flash Operations**

Each layer must have **validation gates** (type checking) that prevent incompatible combinations (e.g., trying to fit Android 11 on a p282 board with Hynix RAM).

### Key Directives
1.  **Declarative over Imperative**: Define *what* the hardware is (in YAML configs) and *what* the target state is. Let the engine figure out *how* to get there.
2.  **Zero "Tribal Knowledge"**: Encode community knowledge (e.g., "Samsung DDR needs timing X") into the `Compatibility Matrix` (code/database), not just documentation or forum posts.
3.  **Safety First**: Use Rust's type system to enforce safety where possible.

## The Golden Rule: Vault Before Flash

> **Every operation begins with an encrypted backup of the original firmware.**

This is non-negotiable. We are dealing with devices that often have unique, per-device data:
- MAC addresses
- WiFi calibration data (NVRAM)
- HDCP keys
- DRM provisioning

**Rule**: No liberation without restoration capability. Always use `phoenix vault` to secure these artifacts before any write operation.

## The Three-Layer Architecture

### 1. The Hardware Liberation Engine (HLE)
*Purpose: Turn a "brick" into a "blank canvas" safely.*
- **SoC Sherlock**: Forensic analysis of the hardware (USB/UART signals). Identifies p281 vs p282, RAM vendors, etc.
- **BootROM Negotiator**: Handles low-level protocols (Amlogic USB Burning, Rockchip Maskrom, Allwinner FEL).
- **The Vault**: Manages encrypted backups.

### 2. The Foundry (Image Build Pipeline)
*Purpose: Transform hardware into purpose-specific infrastructure.*
- **Intent-Based Builds**: Selects OS, Kernel, and DTB based on user intent.
- **Compatibility Matrix**: The rule engine that blocks invalid combinations (e.g., `IF p282 + Hynix + Android 11 → BLOCK`).
- **Patch Engine**: Applies binary patches, DTB overlays, and blob injections.

### 3. The Colony (Mesh Network)
*Purpose: Connect liberated nodes into resilient infrastructure.*
- **Reclamation**: Nodes join a mesh network, report telemetry, and receive OTA updates.

## The Phoenix Methodology: "The Five Rites"

All code changes should respect this workflow:

1.  **Reconnaissance** (`phoenix forensics`): Identify the hardware deeply (RAM, PCB, eMMC).
2.  **Reservation** (`phoenix vault`): Backup everything.
3.  **Restoration** (`phoenix check` → `phoenix patch`): Plan the liberation path using the Compatibility Matrix.
4.  **Rebirth** (`phoenix flash`): Execute the flash operation atomically.
5.  **Reclamation** (`phoenix validate`): Verify success with hardware-in-loop tests.

## Technical Stack Choices

-   **Rust**: For memory safety in raw block device operations and cross-platform binaries.
-   **Tauri 2.0**: For a secure, lightweight GUI.
-   **YAML**: For human-readable, diff-able hardware definitions.
-   **SQLite**: For the embedded Compatibility Matrix database.
