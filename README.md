# Sticky

A lightweight Linux note manager with a terminal UI for browsing notes and a GTK editor for creating and editing them.

## Features

- Terminal-based note manager
- Keyboard-driven TUI
- Create, edit, and delete notes
- GTK4 note editor
- Markdown support
- Local note storage
- Stable UUID-based note identification
- Note types
- Automatic refresh of the note list
- Recent-note ordering
- Alacritty launcher for opening Sticky in a separate terminal

## Tech Stack

- Rust 2024
- Ratatui
- Crossterm
- GTK4
- GLib
- Serde / Serde JSON
- TOML
- UUID
- Chrono
- Clap
- Markdown
- Regex

## Requirements

- Linux
- Arch Linux / Omarchy recommended
- Rust and Cargo
- GTK4
- pkg-config
- Alacritty

## Folder Structure

```text
sticky/
├── .gitignore
├── Cargo.lock
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs
    ├── bin/
    │   ├── note/
    │   │   ├── app.rs
    │   │   ├── editor.rs
    │   │   ├── images.rs
    │   │   ├── mod.rs
    │   │   ├── resizable_image.rs
    │   │   ├── toolbar.rs
    │   │   └── window.rs
    │   ├── stick-note.rs.save
    │   ├── sticky-note.rs
    │   └── sticky.rs
    ├── cli/
    │   └── mod.rs
    ├── config/
    │   ├── mod.rs
    │   └── schema.rs
    ├── manager/
    │   └── mod.rs
    ├── model/
    │   ├── mod.rs
    │   ├── note.rs
    │   └── note_type.rs
    ├── storage/
    │   ├── frontmatter.rs
    │   ├── mod.rs
    │   └── repository.rs
    └── tui/
        ├── app.rs
        ├── events.rs
        ├── mod.rs
        └── ui.rs
```

## Installation on Omarchy

Omarchy is Arch Linux based, so install the required packages with `pacman`.

### 1. Install dependencies

```bash
sudo pacman -Syu
sudo pacman -S --needed base-devel rust gtk4 pkgconf alacritty
```

Verify:

```bash
rustc --version
cargo --version
pkg-config --modversion gtk4
alacritty --version
```

### 2. Clone the repository

```bash
git clone https://github.com/A56-A5/sticky.git
cd sticky
```

### 3. Build Sticky

```bash
cargo clean
cargo build --release
```

The compiled binaries are:

```text
target/release/sticky
target/release/sticky-note
```

## Global Installation

The global setup uses two names:

```text
/usr/local/bin/sticky
        -> launcher script

/usr/local/bin/sticky-bin
        -> actual Rust TUI binary
```

### 1. Install the compiled binaries

```bash
sudo install -Dm755 target/release/sticky /usr/local/bin/sticky-bin
sudo install -Dm755 target/release/sticky-note /usr/local/bin/sticky-note
```

### 2. Create the Sticky launcher

The launcher opens Sticky in a new Alacritty terminal and detaches it from the terminal that launched it.

```bash
sudo tee /usr/local/bin/sticky > /dev/null <<'LAUNCHER'
#!/bin/bash
setsid -f alacritty --class sticky-tui -e /usr/local/bin/sticky-bin
LAUNCHER
```

Make it executable:

```bash
sudo chmod +x /usr/local/bin/sticky
```

### 3. Verify the installation

```bash
which sticky
which sticky-bin
which sticky-note
```

Expected:

```text
/usr/local/bin/sticky
/usr/local/bin/sticky-bin
/usr/local/bin/sticky-note
```

## Run Sticky

Simply run:

```bash
sticky
```

This opens a new Alacritty window containing the Sticky TUI.

The launcher uses `setsid -f`, so the new Sticky terminal is independent of the terminal that launched it. Closing the original terminal does not close Sticky.

## Updating Sticky

From the repository:

```bash
cd ~/Documents/sticky
git pull
```

Rebuild:

```bash
cargo clean
cargo build --release
```

Replace the installed binaries:

```bash
sudo install -Dm755 target/release/sticky /usr/local/bin/sticky-bin
sudo install -Dm755 target/release/sticky-note /usr/local/bin/sticky-note
```

The launcher at `/usr/local/bin/sticky` does not need to be recreated.

Run the updated version:

```bash
sticky
```

## Development

Run the TUI directly from the repository:

```bash
cargo run --bin sticky
```

Run the GTK note editor:

```bash
cargo run --bin sticky-note
```

Build a release version:

```bash
cargo build --release
```

Format the code:

```bash
cargo fmt
```

Check the project:

```bash
cargo check
```

Run tests:

```bash
cargo test
```

## Repository

https://github.com/A56-A5/sticky
