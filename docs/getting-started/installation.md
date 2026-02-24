# Installation

## System Requirements

| Requirement | Minimum |
|---|---|
| OS | Windows 10+, macOS 11+, Linux (X11 or Wayland) |
| GPU | OpenGL 3.3 or Vulkan-capable |
| RAM | 256 MB free |
| Disk | ~20 MB |

## Download

Download the latest release for your platform from the [Releases page](https://github.com/fognil/TaleNode/releases).

| Platform | File |
|---|---|
| Windows | `talenode-windows.zip` |
| macOS | `talenode-macos.dmg` |
| Linux | `talenode-linux.tar.gz` |

Extract the archive and run the `talenode` executable. No installation required.

## Build from Source

TaleNode is written in Rust. You need the Rust toolchain (stable, edition 2021).

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Clone and Build

```bash
git clone https://github.com/fognil/TaleNode.git
cd TaleNode
cargo build --release
```

The binary will be at `target/release/talenode`.

### 3. Run

```bash
cargo run --release
```

!!! note "Debug vs Release"
    Use `cargo run` (without `--release`) during development. The release build enables optimizations that improve rendering performance at higher node counts.

## Dependencies

TaleNode uses these Rust crates (managed automatically by Cargo):

| Crate | Purpose |
|---|---|
| `eframe` + `egui` 0.31 | UI framework |
| `serde` + `serde_json` | Serialization |
| `uuid` | Unique identifiers |
| `rfd` | Native file dialogs |

No external runtime dependencies are required.

## Linux Notes

On Linux, you may need to install development packages for the display server:

=== "Debian/Ubuntu"

    ```bash
    sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev
    ```

=== "Fedora"

    ```bash
    sudo dnf install libxcb-devel libxkbcommon-devel
    ```

## Verify Installation

Launch TaleNode. You should see:

- A dark canvas with a grid background
- A menu bar at the top (File, Edit, View)
- A left panel for Variables, Characters, and Groups
- A status bar at the bottom showing "0 nodes | 0 connections | 100%"

If the window opens correctly, you're ready to go. Head to the [Quick Start](quickstart.md) guide.
