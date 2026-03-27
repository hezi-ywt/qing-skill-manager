# Issue #16 Solution / Issue #16 解决方案

## English

### Summary
This change fixes two marketplace gaps at the same time:
- marketplace search misses curated skills that users expect to find
- users cannot directly download a skill after obtaining a repository or archive URL

### Root Cause
The original download flow mainly assumed a plain GitHub repository URL and then tried to guess the extracted skill folder from the skill name. That is fragile for GitHub tree links that point to a subdirectory, and it does not support a user-pasted source URL as a first-class download path.

### Changes

#### 1. Strengthened backend source parsing
File: `src-tauri/src/utils/download.rs`

- Added unified parsing for three supported source types:
  - GitHub repository URLs
  - GitHub tree URLs with ref and subpath
  - direct ZIP URLs
- Preserved `ref` and `subpath` from GitHub tree URLs and used them during archive extraction.
- Kept public Tauri command signatures unchanged.
- Returned explicit errors for unsupported URLs.

#### 2. Added skills-hub marketplace integration
File: `src-tauri/src/commands/market.rs`

- Added a new market source backed by:
  - `https://raw.githubusercontent.com/qufei1993/skills-hub/main/featured-skills.json`
- Filtered data locally by name, slug, summary, category, and tags.
- Mapped results into the existing `RemoteSkillView` structure.
- Preserved the original GitHub tree link as `source_url` so download behavior is handled by the enhanced parser.
- Reused the existing marketplace aggregation, pagination, and deduplication strategy.

#### 3. Added manual add in the marketplace UI
Files:
- `src/components/ManualAddSkillModal.vue`
- `src/components/MarketPanel.vue`
- `src/composables/useSkillsManager.ts`
- `src/composables/utils.ts`
- `src/App.vue`

- Added a lightweight Manual Add modal.
- Supported URL validation before queueing.
- Added skill name inference rules for GitHub repo, GitHub tree, and ZIP URLs.
- Reused the existing download queue by constructing a virtual remote skill.
- Routed to update when the normalized skill name already exists locally.

#### 4. Updated settings and i18n
Files:
- `src/composables/constants.ts`
- `src/locales/en-US.ts`
- `src/locales/zh-CN.ts`

- Added `skills-hub` as a toggleable market source.
- Hid API Key input for `skills-hub`.
- Added bilingual copy for manual add and validation feedback.

### Validation
- `npm run build` passed.
- Rust unit tests were added, but `cargo test` is still blocked in this environment by current DNS and proxy issues.

## 中文

### 概要
本次改动同时解决了市场能力中的两个缺口：
- 市场搜索覆盖不足，很多精选 Skill 搜不到
- 用户拿到仓库或归档链接后，无法直接在市场页下载 Skill

### 根因
原始下载流程主要假设来源是普通 GitHub 仓库链接，并在解压后依赖技能名去猜测目录。这对于指向子目录的 GitHub tree 链接并不可靠，也无法支持“粘贴来源 URL 直接下载”的产品路径。

### 改动说明

#### 1. 增强后端来源解析
文件：`src-tauri/src/utils/download.rs`

- 新增统一来源解析，支持三类输入：
  - GitHub 仓库链接
  - 带 ref 和 subpath 的 GitHub tree 链接
  - ZIP 直链
- 对 GitHub tree 链接保留 `ref` 与 `subpath`，并在归档解压时优先命中指定子目录。
- Tauri 对外命令签名保持不变。
- 对不支持的 URL 返回明确错误。

#### 2. 接入 skills-hub 市场源
文件：`src-tauri/src/commands/market.rs`

- 新增市场源，读取：
  - `https://raw.githubusercontent.com/qufei1993/skills-hub/main/featured-skills.json`
- 按 name、slug、summary、category、tags 做本地过滤。
- 结果映射到现有 `RemoteSkillView`。
- 保留原始 GitHub tree 链接作为 `source_url`，并交给增强后的下载解析层处理。
- 继续复用现有聚合市场的分页和去重策略。

#### 3. 在市场页新增手动添加
文件：
- `src/components/ManualAddSkillModal.vue`
- `src/components/MarketPanel.vue`
- `src/composables/useSkillsManager.ts`
- `src/composables/utils.ts`
- `src/App.vue`

- 新增轻量级手动添加弹窗。
- 提交前校验 URL 是否属于支持的来源类型。
- 为 GitHub repo、GitHub tree、ZIP URL 增加技能名自动推断规则。
- 前端构造虚拟 RemoteSkill，直接复用现有下载队列。
- 当规范化后的技能名已存在本地时，直接走更新逻辑。

#### 4. 补齐设置项和国际化
文件：
- `src/composables/constants.ts`
- `src/locales/en-US.ts`
- `src/locales/zh-CN.ts`

- 新增可开关的 `skills-hub` 市场源。
- 对 `skills-hub` 隐藏 API Key 输入。
- 补充手动添加和校验提示的中英文文案。

### 验证情况
- `npm run build` 已通过。
- Rust 单测已补充，但当前环境仍受 DNS 和代理问题影响，尚未完成 `cargo test` 验证。
