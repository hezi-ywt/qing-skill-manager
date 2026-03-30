<p align="center">
  <img src="src-tauri/icons/128x128.png" width="80" alt="Qing Skill Manager" />
</p>

<h1 align="center">Qing Skill Manager</h1>

<p align="center">
  <strong>一站式管理所有 AI 编程技能 —— 跨 Agent IDE、跨项目、统一掌控。</strong>
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
  <img src="docs/screenshots/zh-CN/local.jpg" width="720" alt="Skill 库 — 三栏工作台" />
</p>

## 痛点

随着 AI 编程 Agent 越来越多 — Claude Code、Cursor、Codex、OpenCode…… — 散落在你设备上的 **Skill**（提示词文件、自定义指令、Agent 配置）也越来越多。不同 IDE、不同项目、不同版本，没有统一视图。你已经记不清哪个技能在哪了。

**Qing Skill Manager** 给你**一个统一的地方**，看清全局、管理版本、把对的技能推送到对的位置。

## 核心亮点

- **多 IDE 支持** — 在一个应用中管理 Claude Code、Cursor、Codex、OpenCode、OpenClaw 及任意自定义 IDE 的技能
- **统一技能库** — 三栏式工作台：浏览、查看、管理所有技能，支持搜索、筛选和分类（已托管 / 未托管 / 仅插件）
- **智能同步与版本线** — 三方 hash 比较追踪同步状态，彩色标签直观显示。跟随更新或仅复制模式，内置版本线（`main`、`dev`、`stable`），推送/拉取/脱离操作
- **多版本追踪** — 每个技能可有多个版本，结构化对比（标题 vs 正文 vs 资源文件），命名变体，项目级版本锁定
- **按项目部署** — 注册项目、配置 IDE 目标、部署指定版本。自动检测已有技能并解决冲突
- **市场发现** — 一个界面搜索 Claude Plugins、SkillsLLM、SkillsMP，自动检测更新
- **一键收编** — 将手动放置的技能一键导入托管仓库
- **跨平台** — macOS、Windows、Linux

## 界面预览

| Skill 库 | 商店检索 |
|:---:|:---:|
| ![Skill 库](docs/screenshots/zh-CN/local.jpg) | ![商店检索](docs/screenshots/zh-CN/market.jpg) |

| IDE 浏览 | 项目管理 |
|:---:|:---:|
| ![IDE 浏览](docs/screenshots/zh-CN/ide.jpg) | ![项目管理](docs/screenshots/zh-CN/project.jpg) |

## 核心功能

### 多 IDE 技能管理

每个 Agent IDE 都把技能存在自己的目录里。Qing Skill Manager 统一读取它们，清晰展示每个 IDE 里安装了什么，并支持从一个界面安装、卸载或同步。原生支持 **Claude Code、Cursor、Codex、OpenCode、OpenClaw** —— 还可以几秒内注册任意自定义 IDE。

### 智能同步与版本线

每个已安装的技能都通过**三方 hash 比较**（中央 vs 安装时 vs 当前）追踪同步状态，并以彩色标签直观显示：

- **已同步 · main**（绿色）— 与中央一致
- **有更新 · main**（橙色）— 中央有新版本
- **已修改 · main**（蓝色）— 本地有修改未推送
- **冲突 · main**（红色）— 双方都有修改

安装时可选择**跟随更新**（默认，追踪版本线）或**仅复制**（独立，不追踪）模式。内置版本线：`main`、`dev`、`stable`，支持自定义名称。随时通过 ⚙ 按钮修改同步设置。

同步操作：**更新到主库**（推送本地修改）、**获取最新版本**（拉取中央更新）、**停止跟随**（切换为仅复制模式）。

### 多版本技能追踪

每个技能在本地仓库中可以有**多个版本**。支持**结构化对比**（标题 vs 正文 vs 资源文件分层比较，区分"只改了名字"还是"内容真的变了"）。设置默认版本，为不同场景创建命名**变体**（如"精简版" vs "详细版"），并为特定项目锁定指定版本。版本历史记录来源（市场下载、项目导入、手动添加）和创建时间。

### 带分类的技能库

三栏式 **Skill 库**是主要工作台：

- **左侧边栏** — 搜索，按平台筛选（哪个 IDE），按状态筛选（已托管 / 未托管 / 仅插件），按名称或版本数排序
- **中间详情面板** — 选中技能的完整视图：描述、路径、各 IDE 安装状态、项目部署情况、快捷操作
- **右侧版本轨** — 完整版本历史、默认版本标记、每版本的 IDE 和项目计数、重命名/删除/对比/设为默认

技能被分为**已管理**（在你的仓库中追踪）、**未管理**（在某个 IDE 中找到但不在仓库中）和**仅插件**。未管理的技能可以一键"导入并管理"到中央仓库。

### 按项目部署技能

注册你的项目，配置每个项目使用哪些 IDE，然后部署指定版本的技能。应用会**自动检测项目目录中已有的技能**，并在版本不一致时标记冲突。冲突解决提供三个选择：**保留**项目版本、用仓库版本**覆盖**、或**共存**（重命名后两者都保留）。

### 市场发现

在一个界面中搜索 **Claude Plugins**、**SkillsLLM** 和 **SkillsMP** 的技能。下载后直接进入本地仓库，随时可安装到任何位置。版本更新会自动检测。

## 支持的 IDE

| IDE | 全局 Skills 路径 | 项目 Skills 路径 |
|-----|-----------------|-----------------|
| Claude Code | `~/.claude/skills` | `.claude/skills` |
| Codex | `~/.codex/skills` | `.codex/skills` |
| Cursor | `~/.cursor/skills` | `.cursor/skills` |
| OpenClaw | `~/.openclaw/skills` | `.openclaw/skills` |
| OpenCode | `~/.config/opencode/skills` | `.opencode/skills` |

**+ 任意自定义 IDE**（指定名称 + Skills 目录路径即可）。

## 工作原理

```
市场 / 本地文件夹
      ↓ 下载 / 导入
  中央仓库  (~/.qing-skill-manager/skills)
      ↓ 安装（复制）             ↓ 克隆（复制 + 版本锁定）
  全局 IDE 目录                项目级 IDE 目录
  （所有项目可用）              （仅限特定项目）
```

1. **收集** — 从市场下载，或从本地文件夹导入。所有技能统一进入中央仓库。
2. **整理** — 浏览技能库，管理版本，创建变体，分类筛选。
3. **分发** — 全局安装到 IDE，或将特定版本克隆到特定项目。安装时选择同步模式和版本线。
4. **同步** — 应用通过三方 hash 比较追踪每个安装的状态。推送本地修改到中央、拉取最新版本、管理版本线 —— 通过 ⚙ 设置按钮一键操作。

## 快速开始

### 下载

从 [Releases 页面](https://github.com/hezi-ywt/qing-skill-manager/releases) 下载适合你平台的最新版本。

### 从源码构建

**环境要求：** [Node.js](https://nodejs.org/) (LTS)、[Rust](https://rustup.rs/)、[pnpm](https://pnpm.io/)、macOS 需要 Xcode Command Line Tools

```bash
git clone https://github.com/hezi-ywt/qing-skill-manager.git
cd qing-skill-manager/skills-manager
pnpm install
pnpm tauri dev      # 开发模式
pnpm tauri build    # 生产构建
```

### macOS 安全提示

应用尚未配置 Apple 开发者签名。首次启动可能会提示安全警告。执行以下命令即可放行：

```bash
xattr -dr com.apple.quarantine "/Applications/qing-skill-manager.app"
```

## 技术栈

- **桌面端**: [Tauri 2](https://tauri.app/)（Rust 后端 + WebView 前端）
- **前端**: Vue 3 + TypeScript + Vite
- **多语言**: 中文 & English（vue-i18n）

## 数据来源

| 来源 | 地址 |
|------|------|
| Claude Plugins | `https://claude-plugins.dev/api/skills` |
| SkillsLLM | `https://skillsllm.com/api/skills` |
| SkillsMP | `https://skillsmp.com/api/v1/skills/search`（可能需要配置 API Key） |

## 参与贡献

欢迎贡献！请提交 Issue 或 Pull Request。

## 致谢

Qing Skill Manager 基于 [skills-manager](https://github.com/Rito-w/skills-manager) 原始项目继续开发。感谢原作者与所有贡献者。

## 许可证

[MIT](LICENSE)
