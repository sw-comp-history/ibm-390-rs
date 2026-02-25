# Porting Guide: Assembly Game Demos to Public Repos

This document describes the process for extracting assembly game demos from the private `game-lib` monorepo to standalone public repositories.

## Overview

Each assembly game demo (IBM 1130, RCA 1802, IBM 390, RISC-V) follows the same structure and porting process.

## Source and Target Repos

| Demo | Source (Private) | Target (Public) |
|------|------------------|-----------------|
| IBM 1130 | `game-lib/examples/ibm_1130_asm_game` | `sw-comp-history/ibm-1130-rs` |
| RCA 1802 | `game-lib/examples/rca_1802_asm_game` | `sw-comp-history/rca-1802-rs` |
| IBM 390 | `game-lib/examples/ibm_390_asm_game` | `sw-comp-history/ibm-390-rs` |
| RISC-V | `game-lib/examples/rv32_asm_game` | `sw-embed/risc-v-rs` |

## Porting Steps

### 1. Create Target Repository Structure

```
<cpu>-rs/
├── src/                    # Main application source
│   ├── app.rs             # Yew application component
│   ├── lib.rs             # Library root
│   ├── wasm.rs            # WASM bindings
│   ├── assembler.rs       # Assembly parser
│   └── cpu/               # CPU emulation
│       ├── mod.rs
│       ├── state.rs
│       ├── instruction.rs
│       └── executor.rs
├── components/            # Shared UI components (copied)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── components/
├── styles/                # CSS stylesheets
│   ├── asm-game.css      # Shared component styles
│   └── layout.css        # Game-specific layout
├── docs/                  # Documentation
├── images/                # Screenshots
├── index.html             # HTML entry point
├── Trunk.toml             # Trunk build config
├── Cargo.toml             # Workspace config
├── build.rs               # Build-time env vars
├── favicon.ico            # Browser icon
├── LICENSE                # MIT License
└── COPYRIGHT              # Copyright notice
```

### 2. Copy Source Files

From `game-lib/examples/<cpu>_asm_game/`:
- `src/` directory (all Rust source files)
- `index.html` (modify for Trunk)
- CSS files (to `styles/` directory)

From `game-lib/examples/shared-components/`:
- Copy entire directory to `components/`
- Rename package in Cargo.toml to `components`

### 3. Create Workspace Cargo.toml

```toml
[workspace]
members = [".", "components"]
resolver = "2"

[package]
name = "<cpu>-emulator"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
components = { path = "./components" }
yew = { version = "0.21", features = ["csr"] }
wasm-bindgen = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde-wasm-bindgen = "0.6"
web-sys = { version = "0.3", features = ["console"] }
gloo = "0.11"
getrandom = { version = "0.2", features = ["js"] }
console_error_panic_hook = "0.1"
thiserror = "2.0"

[profile.release]
opt-level = "z"
lto = true
```

### 4. Create Trunk.toml

```toml
[build]
dist = "pages"
release = true
public_url = "/<repo-name>/"

[serve]
address = "0.0.0.0"
port = <port>

[watch]
watch = ["src", "index.html", "styles"]
```

### 5. Modify index.html for Trunk

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title><CPU Name> Assembly Game</title>
    <link data-trunk rel="copy-file" href="favicon.ico">
    <link rel="icon" type="image/x-icon" href="favicon.ico">
    <link data-trunk rel="css" href="styles/asm-game.css">
    <link data-trunk rel="css" href="styles/layout.css">
    <link data-trunk rel="rust" href="Cargo.toml" data-wasm-opt="z" />
</head>
<body>
</body>
</html>
```

### 6. Update Import Paths

In `src/app.rs`, change:
```rust
// From:
use asm_game_components::{...};

// To:
use components::{...};
```

### 7. Create build.rs

```rust
use std::process::Command;

fn main() {
    let git_sha = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let timestamp = Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let hostname = Command::new("hostname")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    println!("cargo:rustc-env=VERGEN_GIT_SHA_SHORT={}", git_sha);
    println!("cargo:rustc-env=VERGEN_BUILD_TIMESTAMP={}", timestamp);
    println!("cargo:rustc-env=VERGEN_BUILD_HOST={}", hostname);
    println!("cargo:rerun-if-changed=.git/HEAD");
}
```

### 8. Add UI Enhancements

Add to `src/app.rs`:
- GitHub corner ribbon (top-right)
- Footer with license, copyright, build info

Add to `styles/layout.css`:
- `.github-corner` styles
- `.app-footer` styles

### 9. Fix Reset Button

Ensure Reset callback:
- Calls `hard_reset()` (clears memory)
- Clears `assembly_lines`
- Clears `changed_memory`

### 10. Create GitHub Actions Workflow

`.github/workflows/pages.yml`:
```yaml
name: Deploy to GitHub Pages

on:
  push:
    branches: [gh-pages]

permissions:
  contents: read
  pages: write
  id-token: write

jobs:
  deploy:
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions/configure-pages@v4
      - uses: actions/upload-pages-artifact@v3
        with:
          path: '.'
      - id: deployment
        uses: actions/deploy-pages@v4
```

### 11. Create Documentation

- `docs/architecture.md` - System design
- `docs/prd.md` - Product requirements
- `docs/plan.md` - Implementation roadmap
- `docs/design.md` - Design decisions
- `docs/status.md` - Current status and changelog
- `README.md` - Overview with screenshot, features, references

### 12. Build and Test Locally

```bash
trunk build --release
trunk serve
# Test at http://localhost:<port>
```

### 13. Deploy to GitHub Pages

1. Make repo public
2. Enable GitHub Pages (Settings > Pages > GitHub Actions)
3. Create and push gh-pages branch:
```bash
git checkout -b gh-pages
cp -r pages/* .
git add *.js *.wasm *.css index.html favicon.ico
git commit -m "Initial deployment"
git push -u origin gh-pages
```

### 14. Post-Deployment

- Take screenshot with Playwright
- Update README with screenshot (cache-busting timestamp)
- Add historical references section
- Run clippy and fix warnings
- Update changelog

## Common Code for Future Refactoring

The following code is duplicated across all assembly games and could be extracted to a shared crate:

### 1. Shared UI Components (already shared)
- `components/` crate with Header, Sidebar, Modal, etc.
- `styles/asm-game.css` - common styling

### 2. Potential Shared Infrastructure

**build.rs** - Identical across all games
- Could be a shared build script or proc-macro

**GitHub Corner & Footer** - Same HTML/CSS pattern
- Could be components in the shared crate

**Reset Button Pattern** - Same logic (hard_reset + clear UI state)
- Could be standardized in shared component

**Challenge System** - Similar validation pattern
- Challenge struct and validator could be generic

**Memory Viewer** - Same visualization
- Already shared, but could have more CPU-specific options

### 3. Potential Shared Crate Structure

```
asm-game-common/
├── src/
│   ├── lib.rs
│   ├── build_info.rs      # Build-time env var macros
│   ├── challenge.rs       # Generic challenge system
│   └── ui/
│       ├── github_corner.rs
│       └── footer.rs
```

## Port Checklist

- [ ] Create target repo with empty README
- [ ] Copy src/ files
- [ ] Copy and rename components/
- [ ] Copy styles/ files
- [ ] Create Cargo.toml (workspace)
- [ ] Create Trunk.toml
- [ ] Modify index.html for Trunk
- [ ] Update import paths (asm_game_components -> components)
- [ ] Create build.rs
- [ ] Add GitHub corner to app.rs
- [ ] Add footer to app.rs
- [ ] Add GitHub corner/footer CSS to layout.css
- [ ] Fix Reset button (hard_reset + clear assembly output)
- [ ] Create favicon
- [ ] Create LICENSE and COPYRIGHT
- [ ] Build and test locally
- [ ] Make repo public
- [ ] Enable GitHub Pages
- [ ] Create gh-pages branch and deploy
- [ ] Take screenshot
- [ ] Create docs/ files
- [ ] Update README with screenshot and references
- [ ] Run clippy and fix warnings
- [ ] Final commit and push

## Port Status

| Demo | Status | Live Demo |
|------|--------|-----------|
| IBM 1130 | Complete | https://sw-comp-history.github.io/ibm-1130-rs/ |
| RCA 1802 | Complete | https://sw-comp-history.github.io/rca-1802-rs/ |
| IBM 390 | Not Started | - |
| RISC-V | Not Started | - |
