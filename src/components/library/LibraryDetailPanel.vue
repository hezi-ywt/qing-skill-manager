<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import SyncStatusTag from "../SyncStatusTag.vue";
import type { IdeOption, LibraryIdeInstallation, LibrarySkill, LocalSkill, ProjectConfig } from "../../composables/types";

const { t } = useI18n();

const props = defineProps<{
  skill: LocalSkill | null;
  librarySkill: LibrarySkill | null;
  selectedVersionId: string | null;
  installingId: string | null;
  ideOptions: IdeOption[];
  projects: ProjectConfig[];
}>();

const emit = defineEmits<{
  (e: "install", skill: LocalSkill): void;
  (e: "cloneToProject", projectId: string): void;
  (e: "openDir", path: string): void;
  (e: "delete", skill: LocalSkill): void;
  (e: "adoptToRepo", path: string): void;
  (e: "uninstallSkill", path: string): void;
  (e: "refresh"): void;
}>();

const isInstalling = computed<boolean>(() => {
  return !!props.skill && props.installingId === props.skill.id;
});

const globalInstallations = computed(() => {
  // For managed skills, use installations data
  if (props.librarySkill?.inRepo) {
    const all = props.librarySkill.installations.filter((i) => i.scope === "global");
    if (!props.selectedVersionId) return all;
    return all.filter((i) => i.versionId === props.selectedVersionId);
  }
  // For unmanaged skills, show global sources as installations
  return (props.librarySkill?.unmanagedSources || [])
    .filter((s) => s.scope === "global")
    .map((s) => ({
      ideId: s.ide, ideLabel: s.label, skillPath: s.path,
      versionId: null, isManaged: false, scope: "global" as const,
      syncStatus: "unknown" as const, syncMode: null, syncBranch: null
    }));
});

const pluginInstallations = computed(() =>
  (props.librarySkill?.installations || []).filter((i) => i.scope === "plugin")
);

const projectInstallations = computed(() =>
  (props.librarySkill?.unmanagedSources || [])
    .filter((s) => s.scope === "project")
    .map((s) => ({
      ideId: s.ide, ideLabel: s.label, skillPath: s.path,
      versionId: null, isManaged: false, scope: "project" as const,
      syncStatus: "unknown" as const, syncMode: null, syncBranch: null
    }))
);

const versionProjectMappings = computed(() => {
  const all = props.librarySkill?.projectMappings || [];
  if (!props.selectedVersionId) return all;
  return all.filter((m) => m.versionId === props.selectedVersionId);
});

const selectedVersionName = computed(() => {
  if (!props.selectedVersionId || !props.librarySkill) return null;
  const v = props.librarySkill.versions.find((v) => v.id === props.selectedVersionId);
  return v?.displayName || null;
});


const cloneProjects = computed(() => {
  if (!props.projects.length) {
    return [];
  }

  return props.projects.filter((project) => project.ideTargets.length > 0).map((project) => {
    const mapping = props.librarySkill?.projectMappings.find((item) => item.projectId === project.id) || null;
    return {
      id: project.id,
      name: project.name,
      ideTargets: project.ideTargets,
      mapped: !!mapping && mapping.status !== "missing",
      status: mapping?.status || "missing"
    };
  });
});

function getMappingBadgeClass(status: string): string {
  if (status === "synced") return "success";
  if (status === "conflict") return "danger";
  if (status === "modified") return "warning";
  return "muted";
}

function getMappingLabel(status: string): string {
  if (status === "synced") return t("library.mappingStatusSynced");
  if (status === "conflict") return t("library.mappingStatusConflict");
  if (status === "modified") return t("library.mappingStatusModified");
  return t("library.mappingStatusMissing");
}

function getMappingDescription(status: string): string {
  if (status === "synced") return t("library.mappingDescSynced");
  if (status === "conflict") return t("library.mappingDescConflict");
  if (status === "modified") return t("library.mappingDescModified");
  return t("library.mappingDescMissing");
}

async function handleSyncPush(inst: LibraryIdeInstallation): Promise<void> {
  if (!props.skill) return;
  if (!confirm(t("sync.pushConfirm"))) return;
  try {
    await invoke("sync_push", {
      request: {
        projectSkillPath: inst.skillPath,
        skillId: props.skill.id,
      },
    });
    emit("refresh");
  } catch (e) {
    console.error("sync_push failed:", e);
  }
}

async function handleSyncPull(inst: LibraryIdeInstallation): Promise<void> {
  if (!props.skill) return;
  if (!confirm(t("sync.pullConfirm"))) return;
  try {
    await invoke("sync_pull", {
      request: {
        projectSkillPath: inst.skillPath,
        skillId: props.skill.id,
      },
    });
    emit("refresh");
  } catch (e) {
    console.error("sync_pull failed:", e);
  }
}

async function handleSyncUpdateSettings(inst: LibraryIdeInstallation, syncMode: string, syncBranch: string): Promise<void> {
  try {
    await invoke("sync_update_settings", {
      request: {
        projectSkillPath: inst.skillPath,
        syncMode,
        syncBranch: syncMode === "independent" ? null : (syncBranch || "main"),
      },
    });
    emit("refresh");
  } catch (e) {
    console.error("sync_update_settings failed:", e);
  }
}

// Sync settings popover state
const syncSettingsInst = ref<LibraryIdeInstallation | null>(null);
const syncEditMode = ref<"sync" | "independent">("sync");
const syncEditBranch = ref("main");
const syncCustomBranch = ref("");
const builtinBranches = ["main", "dev", "stable"];

function openSyncSettings(inst: LibraryIdeInstallation) {
  syncSettingsInst.value = inst;
  syncEditMode.value = (inst.syncMode as "sync" | "independent") || "sync";
  const branch = inst.syncBranch || "main";
  if (builtinBranches.includes(branch)) {
    syncEditBranch.value = branch;
    syncCustomBranch.value = "";
  } else {
    syncEditBranch.value = "__custom__";
    syncCustomBranch.value = branch;
  }
}

function closeSyncSettings() {
  syncSettingsInst.value = null;
}

async function confirmSyncSettings() {
  if (!syncSettingsInst.value) return;
  const branch = syncEditBranch.value === "__custom__" ? syncCustomBranch.value : syncEditBranch.value;
  await handleSyncUpdateSettings(syncSettingsInst.value, syncEditMode.value, branch);
  syncSettingsInst.value = null;
}

// Find the project-scope installation matching a project mapping
function getProjectInstallation(mapping: { projectPath: string }) {
  return props.librarySkill?.installations.find(
    (i) => i.scope === "project" && i.skillPath.startsWith(mapping.projectPath + "/")
  ) ?? null;
}

// Open sync settings for a project mapping by finding its installation
function openSyncSettingsForProject(mapping: { projectPath: string; projectName: string; ideTargets: string[] }) {
  const inst = getProjectInstallation(mapping);
  if (inst) {
    openSyncSettings(inst);
  } else {
    // No matching installation found — construct a minimal one for sidecar operations
    const skillName = props.skill?.name || "";
    const skillPath = mapping.projectPath + "/" + skillName;
    openSyncSettings({
      ideId: mapping.ideTargets[0] || "unknown",
      ideLabel: mapping.ideTargets[0] || "unknown",
      skillPath,
      scope: "project",
      isManaged: true,
      versionId: null,
      syncStatus: "unknown",
      syncMode: null,
      syncBranch: null,
    } as LibraryIdeInstallation);
  }
}

// The overall sync status for the skill header tag: take the first sync-mode installation
const primarySyncInstallation = computed<LibraryIdeInstallation | null>(() =>
  props.librarySkill?.installations.find((i) => i.syncMode === "sync") ?? null
);
</script>

<template>
  <main class="library-detail-panel">
    <div v-if="!skill || !librarySkill" class="empty-state">
      <div class="empty-content">
        <h3 class="empty-title">{{ t("library.detail.selectSkill") }}</h3>
        <p class="empty-desc">{{ t("library.detail.selectSkillDesc") }}</p>
      </div>
    </div>

    <div v-else class="skill-detail">
      <div class="detail-header">
        <div class="skill-identity">
          <h1 class="skill-title">
            <span class="repo-dot" :class="librarySkill.inRepo ? 'in-repo' : 'not-in-repo'">{{ librarySkill.inRepo ? "●" : "○" }}</span>
            {{ skill.name }}
          </h1>
          <div class="skill-subtitle">
            <template v-if="librarySkill.inRepo">
              <span v-if="skill.currentVersion" class="version-chip">{{ skill.currentVersion.displayName }}</span>
              <span class="version-meta-text">{{ t("library.detail.versionCount", { count: skill.versionCount }) }}</span>
              <span class="version-meta-text">{{ t("library.usedInProjects", { count: librarySkill.usedByProjectIds.length }) }}</span>
              <SyncStatusTag
                v-if="primarySyncInstallation"
                :sync-status="primarySyncInstallation.syncStatus"
                :sync-mode="primarySyncInstallation.syncMode"
                :sync-branch="primarySyncInstallation.syncBranch"
              />
            </template>
            <template v-else>
              <span class="status-badge unmanaged">{{ t("library.status.unmanaged") }}</span>
            </template>
          </div>
        </div>

        <div class="header-actions">
          <template v-if="librarySkill.inRepo">
            <button class="ghost" @click="$emit('openDir', skill.path)">{{ t("library.detail.openRepoDir") }}</button>
            <button class="ghost danger btn-sm" @click="$emit('delete', skill)">{{ t("library.detail.deleteFromRepo") }}</button>
          </template>
          <template v-else>
            <button class="primary" @click="$emit('adoptToRepo', librarySkill.unmanagedSources[0]?.path || skill.path)">{{ t("library.adoptToRepo") }}</button>
          </template>
        </div>
      </div>

      <section class="panel hero-panel">
        <p class="card-desc">{{ skill.description || t("library.detail.noDescription") }}</p>
        <div class="detail-meta-row">
          <span class="detail-label">{{ t("library.detail.path") }}</span>
          <code class="card-link path-value">{{ librarySkill.displayPath }}</code>
        </div>
        <div v-if="librarySkill.inRepo" class="actions buttons">
          <button class="primary" :disabled="isInstalling" @click="$emit('install', skill)">
            {{ isInstalling ? t("library.detail.installing") : t("library.detail.installToIde") }}
          </button>
          <button v-if="cloneProjects.some((p) => !p.mapped)" class="ghost" @click="$emit('cloneToProject', cloneProjects.find((p) => !p.mapped)!.id)">
            {{ t("library.detail.cloneToProject") }}
          </button>
        </div>
      </section>

      <div v-if="selectedVersionName" class="version-filter-bar">
        <span class="version-filter-label">{{ t("library.filterByVersion") }}</span>
        <span class="version-chip">{{ selectedVersionName }}</span>
      </div>

      <section class="panel section-panel">
        <div class="section-title-row">
          <div class="panel-title section-title-text">{{ t("library.globalInstallations") }}</div>
          <div class="hint">{{ globalInstallations.length > 0 ? t("library.ideCount", { count: globalInstallations.length }) : "" }}</div>
        </div>
        <div v-if="globalInstallations.length === 0" class="hint">{{ selectedVersionId ? t("library.noInstallForVersion") : t("library.notInstalled") }}</div>
        <div v-else class="install-list">
          <div v-for="inst in globalInstallations" :key="inst.skillPath" class="install-entry">
            <div class="install-info">
              <span class="install-ide">{{ inst.ideLabel }}</span>
              <SyncStatusTag
                :sync-status="inst.syncStatus"
                :sync-mode="inst.syncMode"
                :sync-branch="inst.syncBranch"
              />
            </div>
            <div class="install-actions">
              <template v-if="inst.syncMode === 'sync'">
                <button
                  v-if="inst.syncStatus === 'diverged'"
                  class="sync-action-btn push"
                  @click="handleSyncPush(inst)"
                >{{ t("sync.pushToCenter") }}</button>
                <button
                  v-if="inst.syncStatus === 'outdated' || inst.syncStatus === 'conflict'"
                  class="sync-action-btn pull"
                  @click="handleSyncPull(inst)"
                >{{ t("sync.pullLatest") }}</button>
                <span class="action-separator"></span>
              </template>
              <button class="ghost btn-xs sync-settings-btn" @click="openSyncSettings(inst)">⚙</button>
              <button class="ghost btn-xs" @click="$emit('openDir', inst.skillPath)">{{ t("ide.openDir") }}</button>
              <button class="ghost danger btn-xs" @click="$emit('uninstallSkill', inst.skillPath)">{{ t("ide.uninstall") }}</button>
            </div>
          </div>
        </div>
      </section>

      <section class="panel section-panel">
        <div class="section-title-row">
          <div class="panel-title section-title-text">{{ t("library.projectDeployments") }}</div>
          <div class="hint">{{ librarySkill.inRepo ? '' : `${projectInstallations.length} ${t("library.versions.locations")}` }}</div>
        </div>

        <!-- Managed: show project mappings -->
        <template v-if="librarySkill.inRepo">
          <div v-if="versionProjectMappings.length === 0" class="hint">{{ selectedVersionId ? t("library.noProjectForVersion") : t("library.notUsedInProjects") }}</div>
          <div v-else class="mapping-list">
            <article v-for="mapping in versionProjectMappings" :key="mapping.projectId" class="card mapping-card">
              <div class="mapping-header">
                <div class="mapping-title-block">
                  <div class="card-title">{{ mapping.projectName }}</div>
                  <span class="mapping-badge" :class="getMappingBadgeClass(mapping.status)">{{ getMappingLabel(mapping.status) }}</span>
                  <SyncStatusTag
                    v-if="getProjectInstallation(mapping)"
                    :sync-status="getProjectInstallation(mapping)!.syncStatus"
                    :sync-mode="getProjectInstallation(mapping)!.syncMode"
                    :sync-branch="getProjectInstallation(mapping)!.syncBranch"
                  />
                </div>
              </div>
              <div class="mapping-detail">
                <div class="mapping-detail-row">
                  <span class="detail-label">{{ t("library.detail.path") }}</span>
                  <code class="card-link path-value">{{ mapping.projectPath }}</code>
                </div>
                <div class="mapping-detail-row">
                  <span class="detail-label">{{ t("library.versionLabel") }}</span>
                  <span>{{ mapping.versionName || t("library.mappingEmptyVersion") }}</span>
                </div>
                <div class="mapping-detail-row">
                  <span class="detail-label">IDE</span>
                  <span>{{ mapping.ideTargets.join(", ") || "—" }}</span>
                </div>
                <div class="mapping-status-desc hint">{{ getMappingDescription(mapping.status) }}</div>
              </div>
              <div class="mapping-action">
                <template v-if="mapping.status === 'missing'">
                  <button class="ghost btn-sm" @click="$emit('cloneToProject', mapping.projectId)">{{ t("library.actions.clone") }}</button>
                </template>
                <template v-else>
                  <button class="ghost btn-xs sync-settings-btn" @click="openSyncSettingsForProject(mapping)">⚙</button>
                  <button class="ghost btn-xs" @click="$emit('openDir', mapping.projectPath)">{{ t("ide.openDir") }}</button>
                  <button class="ghost danger btn-xs" @click="$emit('uninstallSkill', mapping.projectPath + '/' + (skill?.name || ''))">{{ t("ide.uninstall") }}</button>
                </template>
              </div>
            </article>
          </div>
          <div v-if="!selectedVersionId && cloneProjects.some((p) => !p.mapped)" class="clone-grid">
            <button v-for="project in cloneProjects.filter((p) => !p.mapped)" :key="project.id" class="ghost clone-button" @click="$emit('cloneToProject', project.id)">
              <span>{{ t("library.actions.clone") }} · {{ project.name }}</span>
              <span class="hint">{{ project.ideTargets.join(", ") || t("projects.emptyHint") }}</span>
            </button>
          </div>
        </template>

        <!-- Unmanaged: show project sources -->
        <template v-else>
          <div v-if="projectInstallations.length === 0" class="hint">{{ t("library.notUsedInProjects") }}</div>
          <div v-else class="install-list">
            <div v-for="inst in projectInstallations" :key="inst.skillPath" class="install-entry">
              <div class="install-info">
                <span class="install-ide">{{ inst.ideLabel }}</span>
                <SyncStatusTag
                  :sync-status="inst.syncStatus"
                  :sync-mode="inst.syncMode"
                  :sync-branch="inst.syncBranch"
                />
              </div>
              <div class="install-actions">
                <button class="ghost btn-xs sync-settings-btn" @click="openSyncSettings(inst)">⚙</button>
                <button class="ghost btn-xs" @click="$emit('openDir', inst.skillPath)">{{ t("ide.openDir") }}</button>
                <button class="ghost danger btn-xs" @click="$emit('uninstallSkill', inst.skillPath)">{{ t("ide.uninstall") }}</button>
              </div>
            </div>
          </div>
        </template>
      </section>

      <section v-if="pluginInstallations.length > 0" class="panel section-panel">
        <div class="section-title-row">
          <div class="panel-title section-title-text">{{ t("library.pluginDeployments") }}</div>
          <div class="hint">{{ pluginInstallations.length }} {{ t("library.versions.locations") }}</div>
        </div>
        <div class="install-list">
          <div v-for="inst in pluginInstallations" :key="inst.skillPath" class="install-entry">
            <div class="install-info">
              <span class="install-ide">{{ inst.ideLabel }}</span>
              <span class="mapping-badge muted">{{ t("ide.sourcePlugin") }}</span>
            </div>
            <div class="install-actions">
              <button class="ghost btn-xs" @click="$emit('openDir', inst.skillPath)">{{ t("ide.openDir") }}</button>
            </div>
          </div>
        </div>
      </section>
    </div>

    <!-- Sync Settings Popover -->
    <Teleport to="body">
      <div v-if="syncSettingsInst" class="popover-overlay" @click.self="closeSyncSettings">
        <div class="sync-popover">
          <div class="popover-title">{{ t("sync.editSettings") }}</div>
          <div class="popover-inst-info">
            <span class="popover-ide-name">{{ syncSettingsInst.ideLabel }}</span>
            <span class="popover-path">{{ syncSettingsInst.skillPath }}</span>
          </div>

          <div class="popover-section">
            <div class="popover-label">{{ t("installModal.syncBranch") }}</div>
            <div class="popover-chips">
              <button
                v-for="b in builtinBranches"
                :key="b"
                class="branch-chip"
                :class="{ active: syncEditBranch === b && syncEditMode === 'sync' }"
                :disabled="syncEditMode === 'independent'"
                @click="syncEditBranch = b"
              >{{ b }}</button>
              <button
                class="branch-chip"
                :class="{ active: syncEditBranch === '__custom__' && syncEditMode === 'sync' }"
                :disabled="syncEditMode === 'independent'"
                @click="syncEditBranch = '__custom__'"
              >{{ t("installModal.customBranch") }}</button>
            </div>
            <input
              v-if="syncEditBranch === '__custom__' && syncEditMode === 'sync'"
              v-model="syncCustomBranch"
              class="popover-input"
              :placeholder="t('installModal.customBranchPlaceholder')"
              @keydown.enter="confirmSyncSettings"
            />
          </div>

          <div class="popover-section">
            <div class="popover-label">{{ t("installModal.syncOptions") }}</div>
            <div class="popover-mode-row">
              <label class="mode-option" :class="{ active: syncEditMode === 'sync' }">
                <input type="radio" v-model="syncEditMode" value="sync" />
                {{ t("installModal.syncMode") }}
              </label>
              <label class="mode-option" :class="{ active: syncEditMode === 'independent' }">
                <input type="radio" v-model="syncEditMode" value="independent" />
                {{ t("installModal.independentMode") }}
              </label>
            </div>
          </div>

          <div class="popover-actions">
            <button class="primary btn-sm" @click="confirmSyncSettings">{{ t("sync.confirm") }}</button>
            <button class="ghost btn-sm" @click="closeSyncSettings">{{ t("sync.cancel") }}</button>
          </div>
        </div>
      </div>
    </Teleport>
  </main>
</template>

<style scoped>
.library-detail-panel {
  flex: 1;
  min-width: 0;
  height: 100%;
  overflow-y: auto;
  padding: 0 16px;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}

.empty-content {
  text-align: center;
}

.empty-title {
  margin: 0;
  font-size: 18px;
}

.empty-desc {
  margin-top: 8px;
  color: var(--color-muted);
}

.skill-detail {
  padding: 16px 0;
}

.detail-header {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: flex-start;
  margin-bottom: 16px;
}

.skill-title {
  margin: 0;
  font-size: 24px;
}

.skill-subtitle {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin-top: 8px;
}

.header-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.hero-panel,
.section-panel {
  margin-bottom: 16px;
}

.detail-meta-row {
  display: flex;
  gap: 12px;
  margin-top: 12px;
  align-items: flex-start;
}

.detail-label {
  min-width: 60px;
  color: var(--color-muted);
  font-size: 13px;
}

.path-value {
  margin: 0;
}

.section-title-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: baseline;
  margin-bottom: 12px;
}

.section-title-text {
  margin-bottom: 0;
}

.ide-badges {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.ide-badge {
  padding: 4px 8px;
  border-radius: 999px;
  border: 1px solid var(--color-chip-border);
  background: transparent;
  color: var(--color-meta);
  font-size: 11px;
}

.ide-badge.active {
  border-color: var(--color-success-border);
  background: var(--color-success-bg);
  color: var(--color-success-text);
  font-weight: 600;
}

.mapping-list {
  display: grid;
  gap: 12px;
}

.mapping-card {
  padding: 12px 14px;
}

.mapping-header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: flex-start;
}

.mapping-title-block {
  display: flex;
  align-items: center;
  gap: 10px;
}

.mapping-detail {
  margin-top: 10px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.mapping-detail-row {
  display: flex;
  gap: 10px;
  align-items: baseline;
  font-size: 13px;
}

.mapping-status-desc {
  margin-top: 4px;
  font-size: 12px;
  font-style: italic;
}

.mapping-action {
  margin-top: 8px;
}

.mapping-header .card-title,
.mapping-header .card-meta,
.card-desc,
.path-value,
.clone-button span:first-child,
.clone-button .hint {
  min-width: 0;
  overflow-wrap: anywhere;
}

.mapping-badge {
  padding: 3px 8px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
}

.mapping-badge.success {
  background: var(--color-success-bg);
  border: 1px solid var(--color-success-border);
  color: var(--color-success-text);
}

.mapping-badge.danger {
  background: var(--color-error-bg);
  border: 1px solid var(--color-error-border);
  color: var(--color-error-text);
}

.mapping-badge.warning {
  background: var(--color-chip-bg);
  border: 1px solid var(--color-chip-border);
  color: var(--color-text);
}

.mapping-badge.muted {
  background: var(--color-card-bg);
  border: 1px solid var(--color-card-border);
  color: var(--color-muted);
}

.clone-grid {
  display: grid;
  gap: 10px;
  margin-top: 12px;
}

.clone-button {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: center;
  text-align: left;
}

.clone-button .hint {
  text-align: right;
}

.install-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.install-entry {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 10px;
  border-radius: 8px;
  background: var(--color-card-bg);
  border: 1px solid var(--color-card-border);
}

.install-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.install-ide {
  font-weight: 600;
  font-size: 13px;
}

.install-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.btn-xs {
  padding: 2px 6px;
  font-size: 11px;
}

.repo-dot {
  font-size: 12px;
  margin-right: 6px;
}

.repo-dot.in-repo { color: var(--color-success-text); }
.repo-dot.not-in-repo { color: var(--color-muted); }

.version-filter-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  margin-bottom: 12px;
  border-radius: 8px;
  background: var(--color-chip-bg);
  border: 1px solid var(--color-chip-border);
  font-size: 13px;
}

.version-filter-label {
  color: var(--color-muted);
}

.status-badge.unmanaged {
  background: var(--color-card-bg);
  border: 1px solid var(--color-card-border);
  color: var(--color-muted);
  padding: 3px 8px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
}

.sync-action-btn {
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
  border: none;
  cursor: pointer;
  white-space: nowrap;
}
.sync-action-btn.push {
  background: var(--color-chip-bg);
  color: var(--color-text);
  border: 1px solid var(--color-chip-border);
}
.sync-action-btn.push:hover {
  background: var(--color-chip-border);
}
.sync-action-btn.pull {
  background: var(--color-success-bg);
  color: var(--color-success-text);
  border: 1px solid var(--color-success-border);
}
.sync-action-btn.pull:hover {
  opacity: 0.85;
}
.action-separator {
  width: 1px;
  height: 16px;
  background: var(--color-card-border);
  margin: 0 2px;
}

.sync-settings-btn {
  font-size: 13px;
  opacity: 0.6;
  transition: opacity 0.15s;
}
.sync-settings-btn:hover {
  opacity: 1;
}

/* Sync Settings Popover */
.popover-overlay {
  position: fixed;
  inset: 0;
  z-index: 1100;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.15);
}
.sync-popover {
  background: var(--color-panel-bg, #fff);
  border: 1px solid var(--color-card-border);
  border-radius: 12px;
  padding: 20px;
  width: 320px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
}
.popover-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: 12px;
}
.popover-inst-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-bottom: 16px;
  padding: 8px 10px;
  background: var(--color-card-bg);
  border: 1px solid var(--color-card-border);
  border-radius: 6px;
}
.popover-ide-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text);
}
.popover-path {
  font-size: 11px;
  color: var(--color-muted);
  word-break: break-all;
}
.popover-section {
  margin-bottom: 14px;
}
.popover-label {
  font-size: 12px;
  color: var(--color-muted);
  margin-bottom: 6px;
}
.popover-chips {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}
.branch-chip {
  padding: 4px 12px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 500;
  border: 1px solid var(--color-card-border);
  background: var(--color-card-bg);
  color: var(--color-text);
  cursor: pointer;
  transition: all 0.15s;
}
.branch-chip:hover:not(:disabled) {
  border-color: var(--color-chip-border);
}
.branch-chip.active {
  background: var(--color-success-bg);
  border-color: var(--color-success-border);
  color: var(--color-success-text);
}
.branch-chip:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.popover-input {
  width: 100%;
  margin-top: 8px;
  padding: 6px 10px;
  border: 1px solid var(--color-card-border);
  border-radius: 6px;
  font-size: 12px;
  background: var(--color-bg, #fff);
  color: var(--color-text);
  outline: none;
}
.popover-input:focus {
  border-color: var(--color-success-border);
}
.popover-mode-row {
  display: flex;
  gap: 8px;
}
.mode-option {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 4px 10px;
  border-radius: 6px;
  border: 1px solid var(--color-card-border);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}
.mode-option.active {
  background: var(--color-success-bg);
  border-color: var(--color-success-border);
}
.mode-option input[type="radio"] {
  accent-color: var(--color-success-text);
}
.popover-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  margin-top: 4px;
}
</style>
