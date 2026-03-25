import type { IdeOption, MarketStatus } from "./types";

/**
 * Default IDE options available for skill installation
 * Supports all natively supported IDEs listed in README
 */
export const defaultIdeOptions: IdeOption[] = [
  { id: "antigravity", label: "Antigravity", globalDir: ".gemini/antigravity/skills", projectDir: ".gemini/antigravity/skills" },
  { id: "claude-code", label: "Claude Code", globalDir: ".claude/skills", projectDir: ".claude/skills" },
  { id: "codebuddy", label: "CodeBuddy", globalDir: ".codebuddy/skills", projectDir: ".codebuddy/skills" },
  { id: "codex", label: "Codex", globalDir: ".codex/skills", projectDir: ".codex/skills" },
  { id: "cursor", label: "Cursor", globalDir: ".cursor/skills", projectDir: ".cursor/skills" },
  { id: "kiro", label: "Kiro", globalDir: ".kiro/skills", projectDir: ".kiro/skills" },
  { id: "openclaw", label: "OpenClaw", globalDir: ".openclaw/skills", projectDir: ".openclaw/skills" },
  { id: "opencode", label: "OpenCode", globalDir: ".config/opencode/skills", projectDir: ".opencode/skills" },
  { id: "qoder", label: "Qoder", globalDir: ".qoder/skills", projectDir: ".qoder/skills" },
  { id: "trae", label: "Trae", globalDir: ".trae/skills", projectDir: ".trae/skills" },
  { id: "vscode", label: "VSCode", globalDir: ".github/skills", projectDir: ".github/skills" },
  { id: "windsurf", label: "Windsurf", globalDir: ".windsurf/skills", projectDir: ".windsurf/skills" }
];

/**
 * LocalStorage keys
 */
export const STORAGE_KEYS = {
  IDE_OPTIONS: "qingSkillManager.ideOptions",
  INSTALL_TARGETS: "qingSkillManager.lastInstallTargets",
  MARKET_CONFIGS: "qingSkillManager.marketConfigs",
  ENABLED_MARKETS: "market-enabled",
  PROJECTS: "qingSkillManager.projects"
} as const;

/**
 * Cache time-to-live in milliseconds (10 minutes)
 */
export const CACHE_TTL_MS = 10 * 60 * 1000;

/**
 * Default market statuses
 */
export const defaultMarketStatuses: MarketStatus[] = [
  { id: "claude-plugins", name: "Claude Plugins", status: "online" },
  { id: "skillsllm", name: "SkillsLLM", status: "online" },
  { id: "skillsmp", name: "SkillsMP", status: "needs_key" }
];

/**
 * Default enabled markets
 */
export const defaultEnabledMarkets: Record<string, boolean> = {
  "claude-plugins": true,
  "skillsllm": true,
  "skillsmp": false // Disabled by default until API key is provided
};

/**
 * IDE directory mappings for project-level skills
 * Covers all natively supported IDEs
 */
export const ideDirMappings: Array<{ label: string; path: string; projectPath?: string }> = [
  { label: "Antigravity", path: ".gemini/antigravity/skills" },
  { label: "Claude Code", path: ".claude/skills" },
  { label: "CodeBuddy", path: ".codebuddy/skills" },
  { label: "Codex", path: ".codex/skills" },
  { label: "Cursor", path: ".cursor/skills" },
  { label: "Kiro", path: ".kiro/skills" },
  { label: "OpenClaw", path: ".openclaw/skills" },
  { label: "OpenCode", path: ".config/opencode/skills", projectPath: ".opencode/skills" },
  { label: "Qoder", path: ".qoder/skills" },
  { label: "Trae", path: ".trae/skills" },
  { label: "VSCode", path: ".github/skills" },
  { label: "Windsurf", path: ".windsurf/skills" }
];

export function getProjectIdeRelativeDir(ideLabel: string): string | null {
  const mapping = ideDirMappings.find((item) => item.label === ideLabel);
  if (!mapping) return null;
  return mapping.projectPath || mapping.path;
}

export function buildProjectCloneTargetPath(projectPath: string, ideLabel: string): string | null {
  const relativeDir = getProjectIdeRelativeDir(ideLabel);
  if (!relativeDir) return null;

  if (relativeDir.startsWith("/")) {
    return relativeDir;
  }

  return `${projectPath}/${relativeDir}`;
}
