# Discowl Browser

A modern web browser built on the [Servo](https://servo.org) web engine.

Still in development, you can compile it to test, but security is low, and using it, is at your own risks.
**Only a window appears, UI will be implemented soon**

## Architecture

This browser follows a modular architecture with a clear separation of concerns:

- **app** - The main binary, handles window creation, event loop, and input forwarding
- **engine** - The ONLY crate depending on `servo`. Wraps Servo, manages WebViews, handles rendering
- **browser-core** - Pure Rust browser logic (tabs, history, bookmarks, downloads, settings). No Servo, no winit.
- **ipc** - Shared message types for UI ↔ Rust communication
- **storage** - Persistence layer (SQLite for history/bookmarks/settings)

The UI is implemented as HTML/CSS/JS loaded in dedicated WebViews (toolbar, sidebar, internal pages).

## Building

### Prerequisites

- Rust 1.88+ (managed via `rust-toolchain.toml`)
- System dependencies for Servo (see [Servo build prerequisites](https://servo.org/docs/development/building/))

### Build Commands

```bash
# Debug build (optimized dev profile)
cargo build

# Release build
cargo build --release

# Run
cargo run --bin discowl

# Run release
cargo run --release --bin discowl
```

## Project Structure

```
discowl-browser/
├── Cargo.toml                 # Workspace root
├── rust-toolchain.toml        # Pinned Rust toolchain
├── crates/
│   ├── app/                   # Binary crate
│   ├── engine/                # Servo wrapper (only servo dependency)
│   ├── browser-core/          # Pure Rust browser logic
│   ├── ipc/                   # Message types
│   └── storage/               # Persistence
├── ui/
│   ├── toolbar/               # Toolbar WebView
│   ├── sidebar/               # Sidebar WebView
│   ├── shared/                # Shared JS/CSS
│   └── pages/                 # Internal pages (newtab, settings, error)
├── resources/                 # Icons, assets
└── packaging/                 # Distribution scripts
```

## Development

### Key Concepts

1. **Servo is an engine, not a browser** - You provide the window, tabs, UI, history, etc.
2. **UI is a WebView** - The toolbar, sidebar, and internal pages are HTML loaded in WebViews
3. **One-way data flow** - UI sends commands → Core updates state → Core sends events → UI re-renders
4. **Only `engine` knows Servo types** - Browser-core works with your own types

### Adding Features

1. Add types to `ipc` crate
2. Implement logic in `browser-core`
3. Wire up in `engine` delegate
4. Update UI in `ui/toolbar/` or `ui/sidebar/`

## Configuration

Settings are stored in the platform-appropriate config directory:
- Linux: `~/.config/discowl-browser/`
- macOS: `~/Library/Application Support/discowl-browser/`
- Windows: `%APPDATA%\discowl-browser\`

## License

MPL-2.0