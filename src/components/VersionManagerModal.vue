<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import type { DeleteStrategy, ProjectConfig, ProjectSkill, SkillPackage, SkillVariant, SkillVersion } from "../composables/types";

const props = defineProps<{
  show: boolean;
  skillPackage: SkillPackage | null;
  currentSkillPath?: string;
  selectedSourcePath?: string;
  projects?: ProjectConfig[];
  projectSkills?: ProjectSkill[];
  selectedProjectId?: string | null;
  projectSkillsLoading?: boolean;
  loading: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "rename", versionId: string, newName: string): void;
  (e: "delete", versionId: string, strategy: DeleteStrategy, force: boolean): void;
  (e: "setDefault", versionId: string): void;
  (e: "compare", fromVersionId: string, toVersionId: string): void;
  (e: "createVersion", version: string, displayName: string, sourcePath: string, parentVersion?: string): void;
  (e: "pickSourcePath"): void;
  (e: "pickProject", projectId: string): void;
  (e: "createVariant", versionId: string, name: string, description?: string): void;
  (e: "updateVariant", variantId: string, newName?: string, newVersionId?: string, newDescription?: string): void;
  (e: "deleteVariant", variantId: string): void;
}>();

const { t } = useI18n();

const showRenameModal = ref(false);
const renamingVersion = ref<SkillVersion | null>(null);
const newVersionName = ref("");

const showDeleteModal = ref(false);
const deletingVersion = ref<SkillVersion | null>(null);
const forceDelete = ref(false);
const deleteStrategy = ref<DeleteStrategy>("soft");

const compareFromVersion = ref<string>("");
const compareToVersion = ref<string>("");
const createVersionMode = ref<"folder" | "project">("project");
const createVersionNumber = ref("");
const createVersionDisplayName = ref("");
const createVersionSourcePath = ref("");
const createVersionParentId = ref("");
const projectImportProjectId = ref("");
const projectImportSkillPath = ref("");
const showProjectPicker = ref(false);
const showProjectSkillPicker = ref(false);
const variantName = ref("");
const variantDescription = ref("");
const variantVersionId = ref<string>("");
const editingVariantId = ref<string | null>(null);
const editingVariantName = ref("");
const editingVariantDescription = ref("");
const editingVariantVersionId = ref("");

const sortedVersions = computed(() => {
  if (!props.skillPackage) return [];
  return [...props.skillPackage.versions].sort((a, b) => b.createdAt - a.createdAt);
});

const defaultVersion = computed(() => {
  if (!props.skillPackage) return null;
  return props.skillPackage.versions.find(v => v.id === props.skillPackage?.defaultVersion) || null;
});

const otherVersions = computed(() => {
  if (!props.skillPackage) return [];
  return sortedVersions.value.filter((version) => version.id !== props.skillPackage?.defaultVersion);
});

watch(
  () => props.selectedSourcePath,
  (nextPath) => {
    if (nextPath) {
      createVersionSourcePath.value = nextPath;
    }
  }
);

watch(
  () => props.selectedProjectId,
  (projectId) => {
    if (projectId) {
      projectImportProjectId.value = projectId;
    }
  },
  { immediate: true }
);

watch(projectImportProjectId, (projectId, previous) => {
  if (!projectId || projectId === previous) return;
  projectImportSkillPath.value = "";
  emit("pickProject", projectId);
});

watch(projectImportSkillPath, (skillPath) => {
  if (!skillPath) return;
  createVersionSourcePath.value = skillPath;
});

const selectedProjectSkills = computed(() => props.projectSkills ?? []);
const filteredProjectSkills = computed(() => {
  if (!props.skillPackage) return selectedProjectSkills.value;
  return selectedProjectSkills.value.filter((skill) => skill.name === props.skillPackage?.name);
});

const selectedProject = computed(() => {
  return (props.projects ?? []).find((project) => project.id === projectImportProjectId.value) || null;
});

const selectedProjectSkill = computed(() => {
  return filteredProjectSkills.value.find((skill) => skill.path === projectImportSkillPath.value) || null;
});

const selectedProjectSkillMatchesName = computed(() => {
  if (!selectedProjectSkill.value || !props.skillPackage) return false;
  return selectedProjectSkill.value.name === props.skillPackage.name;
});

const selectedProjectSkillMatchesDefault = computed(() => {
  return selectedProjectSkill.value?.matchesDefaultVersion === true;
});

const selectedProjectSkillMatchedVersionName = computed(() => selectedProjectSkill.value?.matchedVersionName || "");

const selectedProjectSkillIsConflict = computed(() => selectedProjectSkill.value?.status === "conflict");

function handleClose() {
  showRenameModal.value = false;
  showDeleteModal.value = false;
  renamingVersion.value = null;
  deletingVersion.value = null;
  newVersionName.value = "";
  forceDelete.value = false;
  deleteStrategy.value = "soft";
  compareFromVersion.value = "";
  compareToVersion.value = "";
  createVersionNumber.value = "";
  createVersionDisplayName.value = "";
  createVersionSourcePath.value = "";
  createVersionParentId.value = "";
  createVersionMode.value = "project";
  projectImportProjectId.value = props.selectedProjectId || "";
  projectImportSkillPath.value = "";
  showProjectPicker.value = false;
  showProjectSkillPicker.value = false;
  resetVariantEditor();
  emit("close");
}

function chooseProject(projectId: string) {
  projectImportProjectId.value = projectId;
  showProjectPicker.value = false;
}

function chooseProjectSkill(skillPath: string) {
  projectImportSkillPath.value = skillPath;
  showProjectSkillPicker.value = false;
}

function openRenameModal(version: SkillVersion) {
  renamingVersion.value = version;
  newVersionName.value = version.displayName;
  showRenameModal.value = true;
}

function confirmRename() {
  if (renamingVersion.value && newVersionName.value.trim()) {
    emit("rename", renamingVersion.value.id, newVersionName.value.trim());
    showRenameModal.value = false;
    newVersionName.value = "";
    renamingVersion.value = null;
  }
}

function openDeleteModal(version: SkillVersion) {
  deletingVersion.value = version;
  forceDelete.value = false;
  deleteStrategy.value = "soft";
  showDeleteModal.value = true;
}

function confirmDelete() {
  if (deletingVersion.value) {
    emit("delete", deletingVersion.value.id, deleteStrategy.value, forceDelete.value);
    showDeleteModal.value = false;
    deletingVersion.value = null;
    forceDelete.value = false;
    deleteStrategy.value = "soft";
  }
}

function handleSetDefault(versionId: string) {
  emit("setDefault", versionId);
}

function handleCompare() {
  if (compareFromVersion.value && compareToVersion.value) {
    emit("compare", compareFromVersion.value, compareToVersion.value);
  }
}

function handleCreateVariant() {
  if (!variantName.value.trim() || !variantVersionId.value) return;
  emit("createVariant", variantVersionId.value, variantName.value.trim(), variantDescription.value.trim() || undefined);
  variantName.value = "";
  variantDescription.value = "";
  variantVersionId.value = "";
}

function handleCreateVersion() {
  if (!createVersionNumber.value.trim() || !createVersionDisplayName.value.trim() || !createVersionSourcePath.value.trim()) {
    return;
  }
  emit(
    "createVersion",
    createVersionNumber.value.trim(),
    createVersionDisplayName.value.trim(),
    createVersionSourcePath.value.trim(),
    createVersionParentId.value || undefined
  );
  createVersionNumber.value = "";
  createVersionDisplayName.value = "";
  createVersionSourcePath.value = "";
  createVersionParentId.value = "";
}

function handleDeleteVariant(variantId: string) {
  emit("deleteVariant", variantId);
}

function openVariantEditor(variant: SkillVariant) {
  editingVariantId.value = variant.id;
  editingVariantName.value = variant.name;
  editingVariantDescription.value = variant.description || "";
  editingVariantVersionId.value = variant.currentVersion;
}

function resetVariantEditor() {
  editingVariantId.value = null;
  editingVariantName.value = "";
  editingVariantDescription.value = "";
  editingVariantVersionId.value = "";
}

function saveVariantEditor() {
  if (!editingVariantId.value) return;
  emit(
    "updateVariant",
    editingVariantId.value,
    editingVariantName.value.trim() || undefined,
    editingVariantVersionId.value || undefined,
    editingVariantDescription.value.trim() || undefined
  );
  resetVariantEditor();
}

function formatDate(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString();
}

function getSourceLabel(source: string): string {
  const labels: Record<string, string> = {
    market: t("version.sourceMarket"),
    project: t("version.sourceProject"),
    import: t("version.sourceImport"),
    clone: t("version.sourceClone"),
    migration: t("version.sourceMigration")
  };
  return labels[source] || source;
}
</script>

<template>
  <Teleport to="body">
    <div v-if="show" class="modal-overlay" @click.self="handleClose">
      <div class="modal">
        <div class="modal-header">
          <h3>{{ skillPackage?.name ? t("version.title", { name: skillPackage.name }) : t("version.loading") }}</h3>
          <button class="close-btn" @click="handleClose">×</button>
        </div>

        <div class="modal-content">
          <div v-if="loading" class="loading-state">
            {{ t("version.loading") }}
          </div>

          <div v-else-if="!skillPackage || skillPackage.versions.length === 0" class="empty-state">
            {{ t("version.noVersions") }}
          </div>

          <div v-else>
            <section class="hero-section">
              <div class="hero-card">
                <div class="hero-text">
                  <div class="hero-label">{{ t("version.currentVersionSection") }}</div>
                  <div class="hero-title">{{ defaultVersion?.displayName || "-" }}</div>
                  <div class="hero-meta">
                    <span>{{ t("version.defaultVersionId") }}: {{ defaultVersion?.version || "-" }}</span>
                    <span>
                      {{ skillPackage.defaultVersionSource === 'explicit' ? t('version.defaultVersionSourceExplicit') : t('version.defaultVersionSourceStrategy') }}
                    </span>
                    <span>{{ t("version.totalVersions") }}: {{ skillPackage.versions.length }}</span>
                  </div>
                  <p class="hero-help">{{ t("version.currentVersionHelp") }}</p>
                </div>
              </div>
            </section>

            <section class="create-version-section section-card">
              <div class="section-header">
                <div>
                  <h4>{{ t("version.createVersionSimpleTitle") }}</h4>
                  <p class="section-help">{{ t("version.createVersionSimpleHelp") }}</p>
                </div>
              </div>
              <div class="import-mode-switch">
                <button class="ghost" :class="{ active: createVersionMode === 'folder' }" @click="createVersionMode = 'folder'">
                  {{ t("version.importFromFolder") }}
                </button>
                <button class="ghost" :class="{ active: createVersionMode === 'project' }" @click="createVersionMode = 'project'">
                  {{ t("version.importFromProject") }}
                </button>
              </div>

              <div v-if="createVersionMode === 'folder'" class="import-mode-card">
                <div class="mode-title">{{ t("version.importFromFolderTitle") }}</div>
                <p class="section-help">{{ t("version.importFromFolderHelp") }}</p>
                <div class="source-path-row source-input">
                  <input v-model="createVersionSourcePath" class="input source-path-input" :placeholder="t('version.createVersionSourcePathPlaceholder')" />
                  <button class="ghost" @click="$emit('pickSourcePath')">
                    {{ t("version.pickSourcePath") }}
                  </button>
                </div>
              </div>

              <div v-else class="import-mode-card">
                <div class="mode-title">{{ t("version.importFromProjectTitle") }}</div>
                <p class="section-help">{{ t("version.importFromProjectHelp") }}</p>
                <div class="project-import-grid">
                  <div class="picker-shell">
                    <button class="picker-trigger" @click="showProjectPicker = !showProjectPicker">
                      <div class="picker-text">
                        <span class="picker-label">{{ t("version.selectProject") }}</span>
                        <strong>{{ selectedProject?.name || t("version.selectProject") }}</strong>
                        <small v-if="selectedProject" class="picker-subtext">{{ selectedProject.path }}</small>
                      </div>
                      <span class="picker-chevron">▾</span>
                    </button>
                    <div v-if="showProjectPicker" class="picker-panel">
                      <button
                        v-for="project in projects || []"
                        :key="project.id"
                        class="picker-option"
                        :class="{ active: project.id === projectImportProjectId }"
                        @click="chooseProject(project.id)"
                      >
                        <div class="picker-option-title">{{ project.name }}</div>
                        <div class="picker-option-meta">{{ project.path }}</div>
                      </button>
                    </div>
                  </div>

                  <div class="picker-shell">
                    <button class="picker-trigger" :disabled="!projectImportProjectId || projectSkillsLoading" @click="showProjectSkillPicker = !showProjectSkillPicker">
                      <div class="picker-text">
                        <span class="picker-label">{{ t("version.selectProjectSkill") }}</span>
                        <strong>{{ selectedProjectSkill?.name || (projectSkillsLoading ? t("version.loadingProjectSkills") : t("version.selectProjectSkill")) }}</strong>
                        <small v-if="selectedProjectSkill" class="picker-subtext">{{ selectedProjectSkill.path }}</small>
                      </div>
                      <span class="picker-chevron">▾</span>
                    </button>
                    <div v-if="showProjectSkillPicker" class="picker-panel">
                      <button
                        v-for="skill in filteredProjectSkills"
                        :key="skill.path"
                        class="picker-option"
                        :class="{ active: skill.path === projectImportSkillPath }"
                        @click="chooseProjectSkill(skill.path)"
                      >
                        <div class="picker-option-title">{{ skill.name }}</div>
                        <div class="picker-option-meta">{{ skill.path }}</div>
                      </button>
                      <div v-if="!projectSkillsLoading && filteredProjectSkills.length === 0" class="picker-empty-state">
                        {{ t("version.noSameNameProjectSkills") }}
                      </div>
                    </div>
                  </div>
                </div>
                <div v-if="projectImportSkillPath" class="project-import-preview">
                  <span class="preview-label">{{ t("version.sourcePathPreview") }}:</span>
                  <span class="preview-path">{{ projectImportSkillPath }}</span>
                </div>
                <div v-if="selectedProjectSkill" class="project-import-signals">
                  <div v-if="selectedProjectSkillMatchesName" class="signal-chip same-name">
                    {{ t("version.sameNameSkillDetected") }}
                  </div>
                  <div v-if="selectedProjectSkillMatchesDefault" class="signal-chip default-match">
                    {{ t("version.matchesDefaultVersion") }}
                  </div>
                  <div v-else-if="selectedProjectSkillMatchedVersionName" class="signal-chip version-match">
                    {{ t("version.matchesManagedVersion", { name: selectedProjectSkillMatchedVersionName }) }}
                  </div>
                  <div v-if="selectedProjectSkillIsConflict" class="signal-chip conflict">
                    {{ t("version.projectSkillConflictDetected") }}
                  </div>
                </div>
              </div>

              <div class="create-version-grid">
                <input v-model="createVersionNumber" class="input" :placeholder="t('version.createVersionNumberPlaceholder')" />
                <input v-model="createVersionDisplayName" class="input" :placeholder="t('version.createVersionDisplayNamePlaceholder')" />
                <select v-model="createVersionParentId" class="version-select">
                  <option value="">{{ t("version.createVersionParentOptional") }}</option>
                  <option v-for="version in sortedVersions" :key="version.id" :value="version.id">
                    {{ version.displayName }}
                  </option>
                </select>
                <button
                  class="primary"
                  :disabled="!createVersionNumber.trim() || !createVersionDisplayName.trim() || !createVersionSourcePath.trim()"
                  @click="handleCreateVersion"
                >
                  {{ t("version.createVersionConfirm") }}
                </button>
              </div>
            </section>

            <section class="versions-list section-card">
              <div class="section-header">
                <div>
                  <h4>{{ t("version.allVersions") }}</h4>
                  <p class="section-help">{{ t("version.versionListHelp") }}</p>
                </div>
              </div>

              <div v-if="defaultVersion" class="version-item default focused-default">
                <div class="version-header">
                  <div class="version-badges">
                    <span class="badge default-badge">
                      {{ t("version.default") }}
                    </span>
                  </div>
                  <div>
                    <div class="version-name">{{ defaultVersion.displayName }}</div>
                    <div class="version-summary-text">{{ t("version.currentVersionSummary") }}</div>
                  </div>
                </div>

                <div class="version-meta">
                  <div class="meta-row">
                    <span class="meta-label">{{ t("version.defaultVersionId") }}:</span>
                    <span class="meta-value code">{{ defaultVersion.version }}</span>
                  </div>
                  <div class="meta-row">
                    <span class="meta-label">{{ t("version.source") }}:</span>
                    <span class="meta-value">{{ getSourceLabel(defaultVersion.source) }}</span>
                  </div>
                  <div class="meta-row">
                    <span class="meta-label">{{ t("version.createdAt") }}:</span>
                    <span class="meta-value">{{ formatDate(defaultVersion.createdAt) }}</span>
                  </div>
                </div>
              </div>

              <div
                v-for="version in otherVersions"
                :key="version.id"
                class="version-item"
                :class="{ active: version.isActive }"
              >
                <div class="version-header">
                  <div class="version-badges">
                    <span v-if="version.isActive" class="badge active-badge">
                      {{ t("version.active") }}
                    </span>
                  </div>
                  <div>
                    <div class="version-name">{{ version.displayName }}</div>
                    <div class="version-summary-text">{{ t("version.switchToThisVersion") }}</div>
                  </div>
                </div>

                <div class="version-meta">
                  <div class="meta-row">
                    <span class="meta-label">{{ t("version.versionId") }}:</span>
                    <span class="meta-value code">{{ version.version }}</span>
                  </div>
                  <div class="meta-row">
                    <span class="meta-label">{{ t("version.contentHash") }}:</span>
                    <span class="meta-value code">{{ version.contentHash.slice(0, 8) }}</span>
                  </div>
                  <div class="meta-row">
                    <span class="meta-label">{{ t("version.createdAt") }}:</span>
                    <span class="meta-value">{{ formatDate(version.createdAt) }}</span>
                  </div>
                  <div class="meta-row">
                    <span class="meta-label">{{ t("version.source") }}:</span>
                    <span class="meta-value">{{ getSourceLabel(version.source) }}</span>
                  </div>
                </div>

                <div class="version-actions">
                  <button class="primary" @click="handleSetDefault(version.id)">
                    {{ t("version.setAsDefault") }}
                  </button>
                  <button
                    class="ghost"
                    @click="compareFromVersion = skillPackage.defaultVersion; compareToVersion = version.id; handleCompare()"
                  >
                    {{ t("version.compareWithCurrent") }}
                  </button>
                  <button class="ghost" @click="openRenameModal(version)">
                    {{ t("version.rename") }}
                  </button>
                  <button
                    class="ghost danger btn-sm"
                    :disabled="version.id === skillPackage?.defaultVersion"
                    @click="openDeleteModal(version)"
                  >
                    {{ t("version.delete") }}
                  </button>
                </div>
              </div>
            </section>

            <section class="variants-section section-card">
              <div class="section-header">
                <div>
                  <h4>{{ t("version.variants") }}</h4>
                  <p class="section-help">{{ t("version.variantsHelp") }}</p>
                </div>
              </div>
              <div class="variant-create-row">
                <select v-model="variantVersionId" class="version-select">
                  <option value="">{{ t("version.selectVersion") }}</option>
                  <option v-for="version in sortedVersions" :key="version.id" :value="version.id">
                    {{ version.displayName }}
                  </option>
                </select>
                <input v-model="variantName" class="input compact-input" :placeholder="t('version.variantNamePlaceholder')" />
                <input v-model="variantDescription" class="input compact-input" :placeholder="t('version.variantDescriptionPlaceholder')" />
                <button class="primary" :disabled="!variantName.trim() || !variantVersionId" @click="handleCreateVariant">
                  {{ t("version.createVariant") }}
                </button>
              </div>

              <div v-if="skillPackage.variants.length === 0" class="empty-variants">
                {{ t("version.noVariants") }}
              </div>

              <div v-else class="variant-list">
                <div v-for="variant in skillPackage.variants" :key="variant.id" class="variant-item">
                  <div class="variant-main">
                    <div class="variant-name">{{ variant.name }}</div>
                    <div class="variant-meta">{{ variant.description || t("version.noVariantDescription") }}</div>
                    <div class="variant-meta code">{{ variant.currentVersion }}</div>
                  </div>
                  <div class="variant-actions">
                    <button class="ghost" @click="openVariantEditor(variant)">
                      {{ t("version.editVariant") }}
                    </button>
                    <button class="ghost danger btn-sm" @click="handleDeleteVariant(variant.id)">
                      {{ t("version.deleteVariant") }}
                    </button>
                  </div>
                </div>
              </div>

              <div v-if="editingVariantId" class="variant-editor">
                <h5>{{ t("version.editVariantTitle") }}</h5>
                <div class="variant-create-row">
                  <select v-model="editingVariantVersionId" class="version-select">
                    <option value="">{{ t("version.selectVersion") }}</option>
                    <option v-for="version in sortedVersions" :key="version.id" :value="version.id">
                      {{ version.displayName }}
                    </option>
                  </select>
                  <input v-model="editingVariantName" class="input compact-input" :placeholder="t('version.variantNamePlaceholder')" />
                  <input v-model="editingVariantDescription" class="input compact-input" :placeholder="t('version.variantDescriptionPlaceholder')" />
                </div>
                <div class="variant-editor-actions">
                  <button class="ghost" @click="resetVariantEditor">
                    {{ t("common.cancel") }}
                  </button>
                  <button class="primary" :disabled="!editingVariantName.trim() || !editingVariantVersionId" @click="saveVariantEditor">
                    {{ t("version.saveVariant") }}
                  </button>
                </div>
              </div>
            </section>
          </div>
        </div>

        <div class="modal-footer">
          <button class="ghost" @click="handleClose">
            {{ t("common.cancel") }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="showRenameModal" class="submodal-overlay" @click.self="showRenameModal = false">
      <div class="submodal">
        <h4>{{ t("version.renameTitle") }}</h4>
        <input
          v-model="newVersionName"
          type="text"
          class="input"
          :placeholder="t('version.renamePlaceholder')"
          @keyup.enter="confirmRename"
        />
        <div class="submodal-actions">
          <button class="ghost" @click="showRenameModal = false">
            {{ t("common.cancel") }}
          </button>
          <button
            class="primary"
            :disabled="!newVersionName.trim()"
            @click="confirmRename"
          >
            {{ t("version.confirmRename") }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="showDeleteModal" class="submodal-overlay" @click.self="showDeleteModal = false">
      <div class="submodal">
        <h4>{{ t("version.deleteTitle") }}</h4>
        <p class="delete-warning">{{ t("version.deleteWarning", { name: deletingVersion?.displayName }) }}</p>
        <div class="delete-strategy-group">
          <label class="strategy-option">
            <input v-model="deleteStrategy" type="radio" value="soft" />
            <div>
              <strong>{{ t("version.deleteStrategySoft") }}</strong>
              <div class="strategy-desc">{{ t("version.deleteStrategySoftDesc") }}</div>
            </div>
          </label>
          <label class="strategy-option">
            <input v-model="deleteStrategy" type="radio" value="archive" />
            <div>
              <strong>{{ t("version.deleteStrategyArchive") }}</strong>
              <div class="strategy-desc">{{ t("version.deleteStrategyArchiveDesc") }}</div>
            </div>
          </label>
          <label class="strategy-option">
            <input v-model="deleteStrategy" type="radio" value="hard" />
            <div>
              <strong>{{ t("version.deleteStrategyHard") }}</strong>
              <div class="strategy-desc">{{ t("version.deleteStrategyHardDesc") }}</div>
            </div>
          </label>
        </div>
        <label class="checkbox">
          <input v-model="forceDelete" type="checkbox" />
          {{ t("version.forceDelete") }}
        </label>
        <div class="submodal-actions">
          <button class="ghost" @click="showDeleteModal = false">
            {{ t("common.cancel") }}
          </button>
          <button class="primary danger" @click="confirmDelete">
            {{ t("version.confirmDelete") }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1100;
  padding: 20px;
}

.modal {
  background: var(--color-bg);
  border-radius: 12px;
  max-width: 800px;
  width: 100%;
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border);
}

.modal-header h3 {
  margin: 0;
  font-size: 18px;
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: var(--color-muted);
  padding: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
}

.close-btn:hover {
  background: var(--color-hover);
}

.modal-content {
  padding: 20px;
  overflow-y: auto;
  flex: 1;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 20px;
  border-top: 1px solid var(--color-border);
}

.loading-state,
.empty-state {
  text-align: center;
  padding: 40px;
  color: var(--color-muted);
}

.hero-section {
  margin-bottom: 20px;
}

.hero-card,
.section-card {
  background: var(--color-card-bg);
  border: 1px solid var(--color-card-border);
  border-radius: 12px;
  padding: 18px;
}

.hero-label {
  font-size: 12px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--color-muted);
  margin-bottom: 8px;
}

.hero-title {
  font-size: 22px;
  font-weight: 700;
  margin-bottom: 10px;
}

.hero-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  font-size: 12px;
  color: var(--color-muted);
}

.hero-help,
.section-help,
.version-summary-text {
  margin: 8px 0 0 0;
  font-size: 13px;
  color: var(--color-muted);
}

.section-header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: flex-start;
  margin-bottom: 14px;
}

.section-header h4 {
  margin: 0;
  font-size: 16px;
}

.version-info {
  background: var(--color-card-bg);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 20px;
}

.info-row {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
}

.info-row:last-child {
  margin-bottom: 0;
}

.info-label {
  color: var(--color-muted);
  font-size: 13px;
}

.info-value {
  font-weight: 600;
  font-size: 13px;
}

.compare-section {
  margin-bottom: 24px;
  padding-bottom: 20px;
  border-bottom: 1px solid var(--color-border);
}

.create-version-section {
  margin-bottom: 24px;
  padding-bottom: 20px;
  border-bottom: 1px solid var(--color-border);
}

.create-version-section h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
}

.create-version-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.import-mode-switch {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}

.import-mode-switch .ghost.active {
  border-color: var(--color-primary-bg);
  color: var(--color-primary-bg);
  background: color-mix(in srgb, var(--color-primary-bg) 8%, transparent);
}

.import-mode-card {
  margin-bottom: 14px;
  padding: 14px;
  border-radius: 10px;
  border: 1px dashed var(--color-card-border);
  background: color-mix(in srgb, var(--color-card-bg) 92%, var(--color-bg) 8%);
}

.mode-title {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 6px;
}

.project-import-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.picker-shell {
  position: relative;
}

.picker-trigger {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 14px 16px;
  border-radius: 12px;
  border: 1px solid var(--color-card-border);
  background: linear-gradient(180deg, color-mix(in srgb, var(--color-card-bg) 92%, var(--color-bg) 8%), var(--color-bg));
  color: var(--color-text);
  text-align: left;
  box-shadow: 0 10px 24px rgba(0, 0, 0, 0.08), inset 0 1px 0 rgba(255, 255, 255, 0.03);
  transition: border-color 0.18s ease, transform 0.18s ease, box-shadow 0.18s ease;
}

.picker-trigger:hover:not(:disabled) {
  border-color: color-mix(in srgb, var(--color-primary-bg) 55%, var(--color-card-border));
  transform: translateY(-1px);
  box-shadow: 0 14px 28px rgba(0, 0, 0, 0.12), inset 0 1px 0 rgba(255, 255, 255, 0.05);
}

.picker-trigger:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.picker-text {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.picker-label {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--color-muted);
}

.picker-subtext {
  color: var(--color-muted);
  font-size: 12px;
  word-break: break-all;
}

.picker-chevron {
  color: var(--color-muted);
  font-size: 14px;
  flex-shrink: 0;
}

.picker-panel {
  position: absolute;
  top: calc(100% + 8px);
  left: 0;
  right: 0;
  z-index: 20;
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 280px;
  overflow: auto;
  padding: 10px;
  border-radius: 14px;
  border: 1px solid var(--color-card-border);
  background: color-mix(in srgb, var(--color-bg) 92%, var(--color-card-bg) 8%);
  box-shadow: 0 18px 44px rgba(0, 0, 0, 0.22);
}

.picker-option {
  width: 100%;
  text-align: left;
  padding: 12px 14px;
  border-radius: 10px;
  border: 1px solid transparent;
  background: color-mix(in srgb, var(--color-card-bg) 92%, transparent 8%);
  color: var(--color-text);
  transition: border-color 0.18s ease, background-color 0.18s ease, transform 0.18s ease;
}

.picker-option:hover {
  border-color: var(--color-card-border);
  background: color-mix(in srgb, var(--color-primary-bg) 8%, var(--color-card-bg));
  transform: translateY(-1px);
}

.picker-option.active {
  border-color: color-mix(in srgb, var(--color-primary-bg) 65%, var(--color-card-border));
  background: color-mix(in srgb, var(--color-primary-bg) 14%, var(--color-card-bg));
}

.picker-option-title {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 4px;
}

.picker-option-meta {
  font-size: 12px;
  color: var(--color-muted);
  word-break: break-all;
}

.picker-empty-state {
  padding: 14px;
  border-radius: 10px;
  background: color-mix(in srgb, var(--color-card-bg) 88%, transparent 12%);
  color: var(--color-muted);
  font-size: 13px;
  text-align: center;
}

.project-import-grid .version-select,
.create-version-grid .version-select,
.compare-controls .version-select,
.variant-create-row .version-select {
  width: 100%;
}

.project-import-preview {
  margin-top: 10px;
  display: flex;
  gap: 8px;
  align-items: flex-start;
  font-size: 12px;
}

.preview-label {
  color: var(--color-muted);
}

.preview-path {
  font-family: monospace;
  word-break: break-all;
}

.project-import-signals {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 10px;
}

.signal-chip {
  padding: 6px 10px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 600;
  border: 1px solid transparent;
}

.signal-chip.same-name {
  background: color-mix(in srgb, var(--color-primary-bg) 12%, transparent);
  color: var(--color-primary-bg);
  border-color: color-mix(in srgb, var(--color-primary-bg) 35%, transparent);
}

.signal-chip.default-match {
  background: color-mix(in srgb, var(--color-success-bg) 60%, transparent);
  color: var(--color-success-text);
  border-color: var(--color-success-border);
}

.signal-chip.version-match {
  background: color-mix(in srgb, var(--color-primary-bg) 12%, transparent);
  color: var(--color-primary-bg);
  border-color: color-mix(in srgb, var(--color-primary-bg) 40%, transparent);
}

.signal-chip.conflict {
  background: color-mix(in srgb, var(--color-warning-bg) 60%, transparent);
  color: var(--color-warning-text);
  border-color: var(--color-warning-border);
}

.source-input {
  grid-column: 1 / -1;
}

.source-path-row {
  display: flex;
  gap: 8px;
  align-items: stretch;
}

.source-path-input {
  flex: 1;
}

.compare-section h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
}

.compare-controls {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.version-select {
  appearance: none;
  -webkit-appearance: none;
  -moz-appearance: none;
  padding: 12px 42px 12px 14px;
  border: 1px solid var(--color-card-border);
  border-radius: 10px;
  background-color: var(--color-bg);
  background-image:
    linear-gradient(45deg, transparent 50%, var(--color-muted) 50%),
    linear-gradient(135deg, var(--color-muted) 50%, transparent 50%),
    linear-gradient(to right, transparent, transparent);
  background-position:
    calc(100% - 18px) calc(50% - 3px),
    calc(100% - 12px) calc(50% - 3px),
    100% 0;
  background-size: 6px 6px, 6px 6px, 2.5em 2.5em;
  background-repeat: no-repeat;
  color: var(--color-text);
  font-size: 14px;
  min-width: 180px;
  box-shadow: 0 1px 0 rgba(255, 255, 255, 0.03), inset 0 1px 2px rgba(0, 0, 0, 0.06);
  transition: border-color 0.18s ease, box-shadow 0.18s ease, transform 0.18s ease, background-color 0.18s ease;
}

.version-select:hover {
  border-color: color-mix(in srgb, var(--color-primary-bg) 45%, var(--color-card-border));
  background-color: color-mix(in srgb, var(--color-bg) 88%, var(--color-card-bg) 12%);
}

.version-select:focus {
  outline: none;
  border-color: var(--color-primary-bg);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-primary-bg) 18%, transparent), inset 0 1px 2px rgba(0, 0, 0, 0.04);
}

.version-select:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  background-color: color-mix(in srgb, var(--color-bg) 80%, var(--color-card-bg) 20%);
}

.version-select option {
  color: var(--color-text);
  background: var(--color-bg);
}

.compare-arrow {
  color: var(--color-muted);
  font-weight: 600;
}

.versions-list h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
}

.version-item {
  background: var(--color-card-bg);
  border: 1px solid var(--color-card-border);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 12px;
}

.version-item.active {
  border-color: var(--color-success-border);
  box-shadow: inset 0 0 0 1px var(--color-success-border);
}

.version-item.default {
  border-color: var(--color-primary-bg);
  box-shadow: inset 0 0 0 1px var(--color-primary-bg);
}

.focused-default {
  margin-bottom: 12px;
  background: color-mix(in srgb, var(--color-card-bg) 92%, var(--color-primary-bg) 8%);
}

.version-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.version-badges {
  display: flex;
  gap: 6px;
}

.badge {
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
}

.default-badge {
  background: var(--color-primary-bg);
  color: var(--color-primary-text);
}

.active-badge {
  background: var(--color-success-bg);
  color: var(--color-success-text);
  border: 1px solid var(--color-success-border);
}

.version-name {
  font-size: 15px;
  font-weight: 600;
}

.version-meta {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 8px;
  margin-bottom: 12px;
  padding: 12px;
  background: var(--color-bg);
  border-radius: 6px;
}

.meta-row {
  display: flex;
  gap: 6px;
}

.meta-label {
  color: var(--color-muted);
  font-size: 12px;
}

.meta-value {
  font-size: 12px;
}

.meta-value.code {
  font-family: monospace;
  background: var(--color-card-bg);
  padding: 2px 6px;
  border-radius: 4px;
}

.version-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.version-actions button {
  padding: 6px 12px;
  font-size: 12px;
}

.variants-section {
  margin-top: 24px;
}

.variants-section h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
}

.variant-create-row {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
  margin-bottom: 16px;
}

.compact-input {
  flex: 1 1 180px;
}

.empty-variants {
  color: var(--color-muted);
  font-size: 13px;
  padding: 12px 0;
}

.variant-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.variant-item {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: flex-start;
  padding: 12px;
  border-radius: 8px;
  background: var(--color-card-bg);
  border: 1px solid var(--color-card-border);
}

.variant-main {
  flex: 1;
}

.variant-name {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 4px;
}

.variant-meta {
  font-size: 12px;
  color: var(--color-muted);
}

.variant-meta.code {
  font-family: monospace;
  margin-top: 4px;
}

.variant-actions {
  display: flex;
  gap: 8px;
}

.variant-editor {
  margin-top: 16px;
  padding: 16px;
  border-radius: 8px;
  border: 1px solid var(--color-border);
  background: var(--color-bg);
}

.variant-editor .version-select {
  flex: 1 1 220px;
}

.variant-editor h5 {
  margin: 0 0 12px 0;
  font-size: 13px;
}

.variant-editor-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 12px;
}

.delete-strategy-group {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin: 16px 0;
}

.strategy-option {
  display: flex;
  gap: 10px;
  align-items: flex-start;
  padding: 10px;
  border-radius: 8px;
  border: 1px solid var(--color-border);
  background: var(--color-card-bg);
}

.strategy-desc {
  font-size: 12px;
  color: var(--color-muted);
  margin-top: 4px;
}

.submodal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1200;
  padding: 20px;
}

.submodal {
  background: var(--color-bg);
  border-radius: 12px;
  padding: 24px;
  max-width: 400px;
  width: 100%;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
}

.submodal h4 {
  margin: 0 0 16px 0;
  font-size: 16px;
}

.submodal .input {
  width: 100%;
  padding: 10px 14px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-size: 14px;
  margin-bottom: 16px;
  background: var(--color-bg);
  color: var(--color-text);
}

.delete-warning {
  color: var(--color-warning-text);
  font-size: 14px;
  margin-bottom: 16px;
}

.submodal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.submodal-actions button {
  padding: 8px 16px;
}

button.danger {
  background: var(--color-error-bg);
  color: var(--color-error-text);
  border-color: var(--color-error-border);
}

button.danger:hover {
  background: var(--color-error-border);
}
</style>
