# Skill 管理系统设计 v2

> 日期：2026-03-28
> 状态：已实现（PR #4）

---

## 一、设计定位

- **目标用户**：个人为主 + 轻量分享
- **支持 IDE**：OpenCode、Cursor、OpenClaw、Claude Code、Codex
- **设计原则**：遵循主流生态模式（清单文件 + 文件系统 + git 版本追踪），渐进增强现有架构
- **UI 原则**：大道至简，用户一眼就知道怎么用
- **命名原则**：adopt 在前端显示为"导入并管理"

---

## 二、版本控制模型

### 混合版本策略

**本地 skill（用户自建/导入）：**
- 保留现有体系：content hash + version label
- content hash = 整个 skill 目录的 tree hash（`skill_content_hash` 实现）
- 每次内容变更自动生成新版本，记录 `parent_version` 形成版本链
- 版本线系统（Variant）：
  - 内置选项：`main`、`dev`、`stable`
  - 支持用户自定义名称

**市场 skill（从 marketplace 安装）：**
- 在安装 sidecar 中追加 git 追踪字段（`git_source`）
- 检查更新时：比对本地 `sha` vs 远程最新 commit sha
- 更新时：拉取新版本，保留旧版本在版本历史中

### 结构化 Diff

比较两个 skill 差异时，三层分区比较：

| 层级 | 比较什么 | 说明 |
|------|---------|------|
| **title** | SKILL.md 的 `name` 字段 | 仅名称变化 |
| **body** | SKILL.md 的 description、其他 frontmatter + 正文内容 | 指令和元数据的实质变更 |
| **资源文件** | scripts/、references/、assets/ 等子目录 | 脚本、模板、参考文档变更 |

变更类型判定：

| change_type | 含义 | 风险 |
|------------|------|------|
| `identical` | 全部一样 | 无 |
| `title_only` | 只有 name 字段变了 | 低 |
| `body_changed` | body 内容变了（含 description 等 frontmatter） | 中/高 |
| `resource_changed` | 只有资源文件变了 | 中/高 |
| `major_change` | body + 资源都变了 | 高 |

---

## 三、同步状态机

### 安装模式

用户安装 skill 到项目时选择（安装弹窗底部"同步选项"区域）：

- **跟随更新（默认）** — 复制文件夹，管理器追踪与中央的关系，选择同步的版本线（默认 main）
- **仅复制** — 复制文件夹，完全脱离，管理器不再关心

### 版本线

安装时可选择同步到哪个版本线：
- 内置选项：`main`、`dev`、`stable`（pill 按钮组）
- 支持自定义版本线名
- 默认同步 `main`

### 状态模型

```
           中央版本线更新
  synced ─────────────────→ outdated
    ↑                          │
    │ 拉取/推送后              │ 用户获取最新版本
    │                          ↓
    ←──────────────────── synced

           项目侧被编辑
  synced ─────────────────→ diverged
                               │
                 ┌─────────────┼─────────────┐
                 ↓             ↓             ↓
         更新到主库       获取最新版本     停止跟随
         (中央更新,       (放弃本地改动,   (变为仅复制模式)
          通知其他项目)    恢复一致)

           两边都改了
  synced ─────────────────→ conflict
                               │
                 ┌─────────────┼─────────────┐
                 ↓             ↓             ↓
         用结构化 diff    强制覆盖为       停止跟随
         辅助合并         中央/本地版本
```

### 状态检测机制

不存额外状态，scan 时实时计算。sidecar 记录**安装时刻的 hash** 作为基准点，三方比较：

| 中央版本线 hash vs 安装时 hash | 项目 hash vs 安装时 hash | 结果 |
|------------------------------|-------------------------|------|
| 相同 | 相同 | **synced** |
| 不同 | 相同 | **outdated**（中央版本线更新了） |
| 相同 | 不同 | **diverged**（项目侧改了） |
| 不同 | 不同 | **conflict**（两边都改了） |

### 自动升级

单版本 skill 的老安装（无 sidecar）在扫描时自动补写 sidecar，升级为 `synced · main` 状态。多版本 skill 的老安装显示为"未追踪"，需要用户通过 ⚙ 手动设置。

### 前端状态标签

SyncStatusTag 组件，pill 样式，颜色编码区分状态：

```
[已同步 · main]         ← 绿色（success）
[已同步 · dev]          ← 绿色
[有更新 · main]         ← 橙色（chip）
[已修改 · main]         ← 蓝色（chip）
[冲突 · main]           ← 红色（error）
[仅复制]                ← 灰色（muted）
[未追踪]                ← 灰色（仅多版本老安装）
```

### 同步设置编辑

每个安装条目右侧有 ⚙ 按钮，点击弹出同步设置弹窗：
- 版本线选择（main / dev / stable / 自定义）
- 同步模式切换（跟随更新 / 仅复制）
- 弹窗打开时从 sidecar 读取当前设置

⚙ 按钮出现在三个位置：
1. Skill 库 → 详情面板 → 全局安装条目
2. Skill 库 → 详情面板 → 项目部署卡片
3. 项目管理 → 展开详情 → 每个 skill 条目

---

## 四、存储架构

### 文件夹复制模型

- 安装 = `copy_dir_recursive` 复制整个 skill 目录到目标 IDE/项目路径
- 不使用 symlink，保证所有 agent 兼容性
- 每个安装副本独立，互不影响

### 目录结构

```
~/.qing-skill-manager/
├── skills/                          # 中央 skill 存储
│   ├── code-review_anthropic/
│   │   ├── SKILL.md
│   │   ├── scripts/
│   │   └── references/
│   └── git-workflow_local/
│       └── SKILL.md
├── versions/                        # 版本元数据
│   └── {skill_id}/
│       └── package.json             # StoredPackageState（含 variants）
├── archive/                         # 旧版本存档
└── config.json                      # 应用配置（含 default_version_strategy）
```

### Sidecar 文件

**版本元数据 sidecar**（`.qing-skill-manager-version.json`，在中央 skill 目录中）：
```json
{
  "version": "1.0.0",
  "display_name": "Code Review v1",
  "source_url": "https://github.com/owner/repo",
  "parent_version": "0.9.0_abc123",
  "deleted": false,
  "created_at": 1711500000
}
```

**安装 sidecar**（`.qing-skill-version.json`，在项目/IDE 安装目录中）：
```json
{
  "version_id": "1.0.0_abc123",
  "content_hash": "sha256...",
  "installed_at": 1711500000,
  "source_skill_id": "code-review_anthropic",
  "sync_mode": "sync",
  "sync_branch": "main",
  "git_source": {
    "repo": "owner/repo",
    "ref": "v1.2.0",
    "sha": "a3f9c12..."
  }
}
```

字段说明：
- `sync_mode`: `"sync"` | `"independent"` — 安装模式
- `sync_branch`: `"main"` | `"dev"` | 自定义名 — 同步目标版本线
- `git_source`: 可选，仅市场 skill 有 — git 来源追踪

---

## 五、安全机制

- **sync_push/pull/detach 路径验证**：验证路径在 home 目录内、不在中央存储内、包含 SKILL.md
- **sync_pull 原子备份**：删除前先备份，复制失败时自动恢复
- **sidecar 写入错误传播**：不再静默丢弃

---

## 六、后端命令（新增）

| 命令 | 功能 |
|------|------|
| `sync_push` | 将项目修改推送到中央存储 |
| `sync_pull` | 从中央拉取最新版本到项目（原子操作） |
| `sync_detach` | 停止跟随，设为仅复制模式 |
| `sync_get_settings` | 读取安装的当前同步设置 |
| `sync_update_settings` | 更新同步模式和版本线 |

---

## 七、前端组件（新增/修改）

| 组件 | 说明 |
|------|------|
| `SyncStatusTag.vue` | 彩色 pill 标签，显示同步状态 + 版本线 |
| `InstallModal.vue` | 新增同步选项区域（跟随更新/仅复制 + 版本线选择） |
| `LibraryDetailPanel.vue` | 新增 ⚙ 设置弹窗、push/pull 按钮、SyncStatusTag |
| `LibraryVersionRail.vue` | 注册版本改为 Modal 弹窗（含版本线选择），来源标签翻译 |
| `ProjectsPanel.vue` | 展开详情新增 ⚙ 设置弹窗，状态标签翻译 |

---

## 八、i18n 更新

- "纳管" → "导入并管理"（所有出现位置）
- 版本来源翻译：market→市场下载、import→手动导入、clone→安装复制、migration→初始版本、project→项目来源
- 项目 skill 状态翻译：conflict→内容不一致、duplicate→已同步、managed_version→已管理版本
- 新增 `sync` 命名空间（状态标签、操作按钮、确认弹窗、设置弹窗）
- 新增 `installModal` 同步选项 keys

---

## 九、测试覆盖

109 个 Rust 测试，包含：
- `test_sync_full_lifecycle` — synced → diverged → conflict 全流程
- `test_sync_detach_writes_independent_mode` — 停止跟随写入验证
- `test_sync_update_settings` — 版本线切换、独立模式切换、自定义版本线
- `test_validate_project_skill_path` — 路径安全验证（4 个 case）
- `test_structured_diff_resource_changed` — 资源文件变更检测
- `test_installed_sidecar_backward_compat` — 旧 sidecar 向后兼容

---

## 十、后续迭代

- [ ] 分享能力（zip 导出、GitHub 发布、marketplace.json 清单）
- [ ] market.rs 下载时写入 `git_source`（需市场 API 返回 git 信息）
- [ ] sync 操作失败时 toast 通知（当前只 console.error）
- [ ] 结构化 diff 视图组件（当前只有后端逻辑，前端未展示）
