<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { IdeOption, LibrarySkill, LocalSkill, ProjectConfig } from "../../composables/types";

const { t } = useI18n();

const props = defineProps<{
  skill: LocalSkill | null;
  librarySkill: LibrarySkill | null;
  installingId: string | null;
  ideOptions: IdeOption[];
  projects: ProjectConfig[];
}>();

defineEmits<{
  (e: "install", skill: LocalSkill): void;
  (e: "cloneToProject", projectId: string): void;
  (e: "openDir", path: string): void;
  (e: "manageVersions", skill: LocalSkill): void;
  (e: "delete", skill: LocalSkill): void;
}>();

const isInstalling = computed<boolean>(() => {
  return !!props.skill && props.installingId === props.skill.id;
});

const linkedIdes = computed(() => {
  if (!props.skill) {
    return [];
  }

  return props.ideOptions.map((option) => ({
    label: option.label,
    active: props.skill?.usedBy.includes(option.label) ?? false
  }));
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
          <h1 class="skill-title">{{ skill.name }}</h1>
          <div class="skill-subtitle">
            <span v-if="skill.currentVersion" class="version-chip">{{ skill.currentVersion.displayName }}</span>
            <span class="version-meta-text">{{ t("library.detail.versionCount", { count: skill.versionCount }) }}</span>
            <span class="version-meta-text">{{ t("library.usedInProjects", { count: librarySkill.usedByProjectIds.length }) }}</span>
          </div>
        </div>

        <div class="header-actions">
          <button class="ghost" :disabled="isInstalling" @click="$emit('openDir', skill.path)">{{ t("library.detail.openDir") }}</button>
          <button class="ghost" @click="$emit('manageVersions', skill)">{{ t("library.detail.versions") }}</button>
          <button class="ghost danger btn-sm" @click="$emit('delete', skill)">{{ t("local.deleteOne") }}</button>
        </div>
      </div>

      <section class="panel hero-panel">
        <p class="card-desc">{{ skill.description || t("library.detail.noDescription") }}</p>
        <div class="detail-meta-row">
          <span class="detail-label">{{ t("library.detail.path") }}</span>
          <code class="card-link path-value">{{ skill.path }}</code>
        </div>
        <div v-if="skill.source" class="detail-meta-row">
          <span class="detail-label">{{ t("library.detail.source") }}</span>
          <span>{{ skill.source }}</span>
        </div>
        <div class="actions buttons">
          <button class="primary" :disabled="isInstalling" @click="$emit('install', skill)">
            {{ isInstalling ? t("library.detail.installing") : t("library.detail.installToIde") }}
          </button>
        </div>
      </section>

      <section class="panel section-panel">
        <div class="section-title-row">
          <div class="panel-title section-title-text">{{ t("library.detail.installationStatus") }}</div>
          <div class="hint">{{ t("library.installedIn") }}</div>
        </div>
        <div class="ide-badges">
          <span v-for="ide in linkedIdes" :key="ide.label" class="ide-badge" :class="{ active: ide.active }">{{ ide.label }}</span>
        </div>
      </section>

      <section class="panel section-panel">
        <div class="section-title-row">
          <div class="panel-title section-title-text">{{ t("library.projectMappings") }}</div>
          <div class="hint">{{ t("library.cloneHint") }}</div>
        </div>
        <div v-if="librarySkill.projectMappings.length === 0" class="hint">{{ t("library.notUsedInProjects") }}</div>
        <div v-else class="mapping-list">
          <article v-for="mapping in librarySkill.projectMappings" :key="mapping.projectId" class="card mapping-card">
            <div class="mapping-header">
              <div>
                <div class="card-title">{{ mapping.projectName }}</div>
                <div class="card-meta">{{ mapping.projectPath }}</div>
              </div>
              <span class="mapping-badge" :class="getMappingBadgeClass(mapping.status)">{{ getMappingLabel(mapping.status) }}</span>
            </div>
            <div class="mapping-meta">
              <span class="version-meta-text">{{ mapping.versionName || t("library.mappingEmptyVersion") }}</span>
              <span class="version-meta-text">{{ t("projects.ideTargets", { count: mapping.ideTargets.length }) }}</span>
            </div>
          </article>
        </div>
        <div class="clone-grid">
          <button v-for="project in cloneProjects" :key="project.id" class="ghost clone-button" @click="$emit('cloneToProject', project.id)">
            <span>{{ t("library.actions.clone") }} · {{ project.name }}</span>
            <span class="hint">{{ project.ideTargets.join(", ") || t("projects.emptyHint") }}</span>
          </button>
        </div>
      </section>
    </div>
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

.mapping-meta {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
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
</style>
