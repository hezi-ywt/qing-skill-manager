<p align="center">
  <img src="src-tauri/icons/128x128.png" width="80" alt="Qing Skill Manager" />
</p>

<h1 align="center">Qing Skill Manager</h1>

<p align="center">
  <strong>A unified desktop app to manage AI coding skills across every Agent IDE and every project.</strong>
</p>

<p align="center">
  <a href="https://github.com/hezi-ywt/qing-skill-manager/releases"><img src="https://img.shields.io/github/v/release/hezi-ywt/qing-skill-manager?style=flat-square" alt="Release" /></a>
  <a href="https://github.com/hezi-ywt/qing-skill-manager/blob/main/LICENSE"><img src="https://img.shields.io/github/license/hezi-ywt/qing-skill-manager?style=flat-square" alt="License" /></a>
  <a href="https://github.com/hezi-ywt/qing-skill-manager/stargazers"><img src="https://img.shields.io/github/stars/hezi-ywt/qing-skill-manager?style=flat-square" alt="Stars" /></a>
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-blue?style=flat-square" alt="Platform" />
  <img src="https://img.shields.io/badge/built%20with-Tauri%202%20%2B%20Vue%203%20%2B%20Rust-orange?style=flat-square" alt="Tech Stack" />
</p>

<p align="center">
  <a href="README.md">English</a> | <a href="README_zh-CN.md">中文</a>
</p>

<p align="center">
  <img src="docs/screenshots/en-US/local.jpg" width="720" alt="Skill Library — three-column workspace" />
</p>

## The Problem

As AI-powered coding agents multiply — Claude Code, Cursor, Codex, OpenCode, and more — so do the **skills** (prompt files, custom instructions, agent configurations) scattered across your device. Different IDEs, different projects, different versions, no central view. You've lost track of what's where.

**Qing Skill Manager** gives you **one place** to see everything, keep it organized, and push the right version to the right place.

## Key Features

- **Multi-IDE support** — Manage skills for Claude Code, Cursor, Codex, OpenCode, OpenClaw, and any custom IDE from a single app
- **Unified skill library** — Three-column workspace: browse, inspect, and manage all your skills with search, filtering, and classification (Managed / Unmanaged / Plugin-Only)
- **Multi-version tracking** — Each skill can have multiple versions with side-by-side diffs, similarity scores, named variants, and per-project version pinning
- **Per-project deployment** — Register projects, configure IDE targets, and deploy specific skill versions. Auto-detects existing skills and resolves conflicts
- **Marketplace discovery** — Search and download from Claude Plugins, SkillsLLM, and SkillsMP in one interface, with automatic update detection
- **Adopt unmanaged skills** — One-click import of manually placed skills into your managed repository
- **Cross-platform** — macOS, Windows, and Linux

## Screenshots

| Skill Library | Marketplace |
|:---:|:---:|
| ![Skill Library](docs/screenshots/en-US/local.jpg) | ![Marketplace](docs/screenshots/en-US/market.jpg) |

| IDE Browser | Projects |
|:---:|:---:|
| ![IDE Browser](docs/screenshots/en-US/ide.jpg) | ![Projects](docs/screenshots/en-US/project.jpg) |

## Supported IDEs

| IDE | Global Skills Path | Project Skills Path |
|-----|-------------------|-------------------|
| Claude Code | `~/.claude/skills` | `.claude/skills` |
| Codex | `~/.codex/skills` | `.codex/skills` |
| Cursor | `~/.cursor/skills` | `.cursor/skills` |
| OpenClaw | `~/.openclaw/skills` | `.openclaw/skills` |
| OpenCode | `~/.config/opencode/skills` | `.opencode/skills` |

**+ Any custom IDE** you register (name + skills directory path).

## How It Works

```
Marketplace / Local folder
        ↓ download / import
  Central Repository  (~/.qing-skill-manager/skills)
        ↓ install (copy)           ↓ clone (copy + version pin)
  Global IDE directories        Project IDE directories
  (available everywhere)        (scoped to one project)
```

1. **Collect** — Download from marketplaces, or import from local folders
2. **Organize** — Browse your library, manage versions, create variants
3. **Distribute** — Install globally to IDEs, or clone specific versions into projects
4. **Maintain** — Track what's installed where, detect conflicts, and resolve them

## Quick Start

### Download

Grab the latest release for your platform from the [Releases page](https://github.com/hezi-ywt/qing-skill-manager/releases).

### Build from Source

**Prerequisites:** [Node.js](https://nodejs.org/) (LTS), [Rust](https://rustup.rs/), [pnpm](https://pnpm.io/), macOS: Xcode Command Line Tools

```bash
git clone https://github.com/hezi-ywt/qing-skill-manager.git
cd qing-skill-manager/skills-manager
pnpm install
pnpm tauri dev      # Development
pnpm tauri build    # Production build
```

### macOS Security Note

The app is not yet signed with an Apple Developer certificate. On first launch you may see a security warning. Run:

```bash
xattr -dr com.apple.quarantine "/Applications/qing-skill-manager.app"
```

## Tech Stack

- **Desktop**: [Tauri 2](https://tauri.app/) (Rust backend, WebView frontend)
- **Frontend**: Vue 3 + TypeScript + Vite
- **i18n**: English & Simplified Chinese (vue-i18n)

## Data Sources

| Source | URL |
|--------|-----|
| Claude Plugins | `https://claude-plugins.dev/api/skills` |
| SkillsLLM | `https://skillsllm.com/api/skills` |
| SkillsMP | `https://skillsmp.com/api/v1/skills/search` (API key may be required) |

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## Acknowledgement

Qing Skill Manager is built on top of the original [skills-manager](https://github.com/Rito-w/skills-manager). Thanks to the original author and all contributors.

## License

[MIT](LICENSE)
