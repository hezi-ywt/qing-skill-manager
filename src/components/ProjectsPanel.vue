<script setup lang="ts">
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { ref } from "vue";
import type { ProjectConfig, LocalSkill, IdeOption, ProjectSkill } from "../composables/types";
import { useI18n } from "vue-i18n";

const { t } = useI18n();

const props = defineProps<{
  projects: ProjectConfig[];
  selectedProjectId: string | null;
  localSkills: LocalSkill[];
  ideOptions: IdeOption[];
  projectSkillSnapshots?: Record<string, ProjectSkill[]>;
  localLoading: boolean;
}>();

const expandedProjectId = ref<string | null>(null);

const emit = defineEmits<{
  (e: "addProject"): void;
  (e: "removeProject", projectId: string): void;
  (e: "selectProject", projectId: string | null): void;
  (e: "configureProject", projectId: string): void;
  (e: "exportSkills", projectId: string): void;
  (e: "importSkills", projectId: string): void;
}>();

function handleAddProject() {
  emit("addProject");
}

function handleRemoveProject(projectId: string) {
  emit("removeProject", projectId);
}

function handleSelectProject(projectId: string) {
  emit("selectProject", projectId === props.selectedProjectId ? null : projectId);
}

function handleConfigureProject(projectId: string) {
  emit("configureProject", projectId);
}

function handleExportSkills(projectId: string) {
  emit("exportSkills", projectId);
}

function handleImportSkills(projectId: string) {
  emit("importSkills", projectId);
}

async function handleOpenDirectory(project: ProjectConfig) {
  try {
    await revealItemInDir(project.path);
  } catch (err) {
    console.error("Failed to open directory:", err);
  }
}

function buildIdeBadgeList(project: ProjectConfig) {
  return project.ideTargets.map((ideLabel) => ({
    label: ideLabel,
    active: true
  }));
}

function getProjectSkillStats(projectId: string) {
  const skills = props.projectSkillSnapshots?.[projectId] ?? [];
  const conflicts = skills.filter((skill) => skill.status === "conflict").length;
  const duplicates = skills.filter((skill) => skill.status === "duplicate").length;
  const managedVersions = skills.filter((skill) => skill.status === "managed_version").length;
  const sameNameDefaults = skills.filter((skill) => skill.matchesDefaultVersion === true).length;
  const versionMatches = skills.filter((skill) => !!skill.matchedVersionId && skill.matchesDefaultVersion !== true).length;
  return { conflicts, duplicates, managedVersions, sameNameDefaults, versionMatches, total: skills.length };
}

function toggleProjectDetails(projectId: string) {
  expandedProjectId.value = expandedProjectId.value === projectId ? null : projectId;
}

function getProjectSkillDetails(projectId: string) {
  const skills = props.projectSkillSnapshots?.[projectId] ?? [];
  return skills.filter((skill) => skill.status === "conflict" || skill.status === "duplicate" || skill.status === "managed_version");
}
</script>

<template>
  <section class="panel">
    <div class="panel-header">
      <div class="panel-title">{{ t("projects.title") }}</div>
      <button class="primary" @click="handleAddProject">
        {{ t("projects.add") }}
      </button>
    </div>
    <div class="hint">{{ t("projects.hint") }}</div>

    <div v-if="projects.length === 0" class="hint">{{ t("projects.emptyHint") }}</div>

    <div v-else class="project-list">
      <div
        v-for="project in projects"
        :key="project.id"
        class="project-item"
        :class="{ selected: selectedProjectId === project.id }"
      >
        <div class="project-header">
          <div class="project-info">
            <div class="project-name">{{ project.name }}</div>
            <div class="project-path">{{ project.path }}</div>
          </div>
          <div class="project-actions">
            <button
              class="ghost small"
              @click="handleSelectProject(project.id)"
            >
              {{ selectedProjectId === project.id ? t("projects.deselect") : t("projects.select") }}
            </button>
            <button
              class="ghost small"
              @click="handleConfigureProject(project.id)"
            >
              {{ t("projects.configure") }}
            </button>
            <button
              class="ghost small"
              @click="handleOpenDirectory(project)"
            >
              {{ t("projects.openDirectory") }}
            </button>
            <button
              class="primary small"
              @click="handleExportSkills(project.id)"
            >
              {{ t("projects.scanProjectSkills") }}
            </button>
            <button
              class="primary small"
              :disabled="localLoading"
              @click="handleImportSkills(project.id)"
            >
              {{ t("projects.cloneSkillsToProject") }}
            </button>
            <button
              class="ghost danger small"
              @click="handleRemoveProject(project.id)"
            >
              {{ t("projects.remove") }}
            </button>
          </div>
        </div>
        <div class="project-meta">
          <span class="meta-item">
            {{ t("projects.ideTargets", { count: project.ideTargets.length }) }}
          </span>
          <span v-if="project.detectedIdeDirs.length > 0" class="meta-item">
            {{ t("projects.detected", { count: project.detectedIdeDirs.length }) }}
          </span>
          <span v-if="getProjectSkillStats(project.id).conflicts > 0" class="meta-item warning">
            {{ t("projects.conflictSkills") }}: {{ getProjectSkillStats(project.id).conflicts }}
          </span>
          <span v-if="getProjectSkillStats(project.id).duplicates > 0" class="meta-item info">
            {{ t("projects.modifiedOrSameNameSkills") }}: {{ getProjectSkillStats(project.id).duplicates }}
          </span>
          <span v-if="getProjectSkillStats(project.id).managedVersions > 0" class="meta-item success">
            {{ t("projects.importedManagedVersions") }}: {{ getProjectSkillStats(project.id).managedVersions }}
          </span>
          <span v-if="getProjectSkillStats(project.id).sameNameDefaults > 0" class="meta-item success">
            {{ t("projects.defaultMatchedSkills") }}: {{ getProjectSkillStats(project.id).sameNameDefaults }}
          </span>
          <span v-if="getProjectSkillStats(project.id).versionMatches > 0" class="meta-item info">
            {{ t("projects.otherMatchedVersions") }}: {{ getProjectSkillStats(project.id).versionMatches }}
          </span>
        </div>
        <div class="ide-badges">
          <span
            v-for="badge in buildIdeBadgeList(project)"
            :key="badge.label"
            class="ide-badge"
            :class="{ active: badge.active }"
          >
            {{ badge.label }}
          </span>
        </div>
        <div v-if="getProjectSkillStats(project.id).total > 0" class="project-detail-toggle-row">
          <button class="ghost small" @click="toggleProjectDetails(project.id)">
            {{ expandedProjectId === project.id ? t("projects.hideSkillDetails") : t("projects.showSkillDetails") }}
          </button>
        </div>
        <div v-if="expandedProjectId === project.id" class="project-skill-details">
          <div v-if="getProjectSkillDetails(project.id).length === 0" class="detail-empty">
            {{ t("projects.noMonitoredSkillChanges") }}
          </div>
          <div v-for="skill in getProjectSkillDetails(project.id)" :key="skill.path" class="detail-item">
            <div class="detail-main">
              <div class="detail-name-row">
                <span class="detail-name">{{ skill.name }}</span>
                <span class="detail-status" :class="skill.status">{{ skill.status }}</span>
              </div>
              <div class="detail-path">{{ skill.path }}</div>
              <div class="detail-badges">
                <span v-if="skill.matchesDefaultVersion === true" class="detail-chip success">
                  {{ t("projects.defaultMatchedSkills") }}
                </span>
                <span v-else-if="skill.status === 'managed_version'" class="detail-chip success">
                  {{ t("projects.importedManagedVersions") }}
                </span>
                <span v-else-if="skill.matchedVersionName" class="detail-chip info">
                  {{ t("projects.matchesManagedVersion", { name: skill.matchedVersionName }) }}
                </span>
                <span v-if="skill.status === 'conflict'" class="detail-chip warning">
                  {{ t("projects.conflictSkills") }}
                </span>
                <span v-if="skill.status === 'duplicate'" class="detail-chip info">
                  {{ t("projects.modifiedOrSameNameSkills") }}
                </span>
                <span v-if="skill.matchedRegistrySkill?.path" class="detail-chip info">
                  {{ t("projects.matchesManagedVersion", { name: skill.matchedRegistrySkill.name }) }}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.panel-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text);
  margin: 0;
}

.project-list {
  margin-top: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.project-item {
  padding: 16px;
  background: var(--color-card-bg);
  border: 1px solid var(--color-card-border);
  border-radius: 8px;
  transition: all 0.2s ease;
}

.project-item.selected {
  border-color: var(--color-primary-bg);
  box-shadow: 0 0 0 2px rgba(0, 113, 227, 0.2);
}

.project-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  flex-wrap: wrap;
}

.project-info {
  flex: 1;
  min-width: 200px;
}

.project-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: 4px;
}

.project-path {
  font-size: 12px;
  color: var(--color-muted);
  word-break: break-all;
}

.project-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.project-actions button {
  padding: 6px 12px;
  font-size: 13px;
}

.project-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  margin-top: 12px;
  font-size: 12px;
  color: var(--color-muted);
}

.meta-item {
  padding: 2px 8px;
  background: var(--color-chip-bg);
  border-radius: 999px;
  font-size: 11px;
}

.meta-item.warning {
  background: var(--color-warning-bg);
  color: var(--color-warning-text);
}

.meta-item.info {
  background: color-mix(in srgb, var(--color-primary-bg) 12%, transparent);
  color: var(--color-primary-bg);
}

.meta-item.success {
  background: var(--color-success-bg);
  color: var(--color-success-text);
}

.ide-badges {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 8px;
}

.ide-badge {
  padding: 4px 8px;
  border-radius: 999px;
  border: 1px solid var(--color-chip-border);
  background: transparent;
  color: var(--color-muted);
  font-size: 11px;
  line-height: 1.2;
}

.ide-badge.active {
  border-color: var(--color-success-border);
  background: var(--color-success-bg);
  color: var(--color-success-text);
  font-weight: 600;
}

.project-detail-toggle-row {
  margin-top: 10px;
}

.project-skill-details {
  margin-top: 12px;
  padding: 12px;
  border-radius: 10px;
  border: 1px solid var(--color-card-border);
  background: color-mix(in srgb, var(--color-card-bg) 92%, var(--color-bg) 8%);
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.detail-empty {
  color: var(--color-muted);
  font-size: 12px;
}

.detail-item {
  padding: 10px 12px;
  border-radius: 8px;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
}

.detail-name-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.detail-name {
  font-size: 14px;
  font-weight: 600;
}

.detail-status {
  text-transform: uppercase;
  font-size: 10px;
  font-weight: 700;
  padding: 3px 7px;
  border-radius: 999px;
  background: var(--color-chip-bg);
  color: var(--color-muted);
}

.detail-status.conflict {
  background: var(--color-warning-bg);
  color: var(--color-warning-text);
}

.detail-status.duplicate {
  background: color-mix(in srgb, var(--color-primary-bg) 12%, transparent);
  color: var(--color-primary-bg);
}

.detail-status.managed_version {
  background: var(--color-success-bg);
  color: var(--color-success-text);
}

.detail-path {
  font-size: 12px;
  color: var(--color-muted);
  word-break: break-all;
}

.detail-badges {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 8px;
}

.detail-chip {
  padding: 4px 8px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
}

.detail-chip.success {
  background: var(--color-success-bg);
  color: var(--color-success-text);
}

.detail-chip.warning {
  background: var(--color-warning-bg);
  color: var(--color-warning-text);
}

.detail-chip.info {
  background: color-mix(in srgb, var(--color-primary-bg) 12%, transparent);
  color: var(--color-primary-bg);
}
</style>
