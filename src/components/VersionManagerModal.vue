<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import type { DeleteStrategy, SkillPackage, SkillVariant, SkillVersion } from "../composables/types";

const props = defineProps<{
  show: boolean;
  skillPackage: SkillPackage | null;
  currentSkillPath?: string;
  selectedSourcePath?: string;
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
const createVersionNumber = ref("");
const createVersionDisplayName = ref("");
const createVersionSourcePath = ref("");
const createVersionParentId = ref("");
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

watch(
  () => props.selectedSourcePath,
  (nextPath) => {
    if (nextPath) {
      createVersionSourcePath.value = nextPath;
    }
  }
);

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
  resetVariantEditor();
  emit("close");
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

function useCurrentVersionAsSource() {
  if (!props.currentSkillPath) return;
  createVersionSourcePath.value = props.currentSkillPath;
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
            <div class="version-info">
              <div class="info-row">
                <span class="info-label">{{ t("version.totalVersions") }}:</span>
                <span class="info-value">{{ skillPackage.versions.length }}</span>
              </div>
              <div class="info-row">
                <span class="info-label">{{ t("version.defaultVersion") }}:</span>
                <span class="info-value">{{ defaultVersion?.displayName || "-" }}</span>
              </div>
            </div>

            <div class="compare-section">
              <h4>{{ t("version.compareVersions") }}</h4>
              <div class="compare-controls">
                <select v-model="compareFromVersion" class="version-select">
                  <option value="">{{ t("version.selectVersion") }}</option>
                  <option v-for="version in sortedVersions" :key="version.id" :value="version.id">
                    {{ version.displayName }}
                  </option>
                </select>
                <span class="compare-arrow">→</span>
                <select v-model="compareToVersion" class="version-select">
                  <option value="">{{ t("version.selectVersion") }}</option>
                  <option v-for="version in sortedVersions" :key="version.id" :value="version.id">
                    {{ version.displayName }}
                  </option>
                </select>
                <button
                  class="primary"
                  :disabled="!compareFromVersion || !compareToVersion || compareFromVersion === compareToVersion"
                  @click="handleCompare"
                >
                  {{ t("version.compare") }}
                </button>
              </div>
            </div>

            <div class="create-version-section">
              <h4>{{ t("version.createVersion") }}</h4>
              <div class="create-version-grid">
                <input v-model="createVersionNumber" class="input" :placeholder="t('version.createVersionNumberPlaceholder')" />
                <input v-model="createVersionDisplayName" class="input" :placeholder="t('version.createVersionDisplayNamePlaceholder')" />
                <div class="source-path-row source-input">
                  <input v-model="createVersionSourcePath" class="input source-path-input" :placeholder="t('version.createVersionSourcePathPlaceholder')" />
                  <button class="ghost" @click="$emit('pickSourcePath')">
                    {{ t("version.pickSourcePath") }}
                  </button>
                  <button v-if="currentSkillPath" class="ghost" @click="useCurrentVersionAsSource">
                    {{ t("version.useCurrentVersion") }}
                  </button>
                </div>
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
            </div>

            <div class="versions-list">
              <h4>{{ t("version.allVersions") }}</h4>
              <div
                v-for="version in sortedVersions"
                :key="version.id"
                class="version-item"
                :class="{ active: version.isActive, default: version.id === skillPackage?.defaultVersion }"
              >
                <div class="version-header">
                  <div class="version-badges">
                    <span v-if="version.id === skillPackage?.defaultVersion" class="badge default-badge">
                      {{ t("version.default") }}
                    </span>
                    <span v-if="version.isActive" class="badge active-badge">
                      {{ t("version.active") }}
                    </span>
                  </div>
                  <div class="version-name">{{ version.displayName }}</div>
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
                  <button
                    v-if="version.id !== skillPackage?.defaultVersion"
                    class="ghost"
                    @click="handleSetDefault(version.id)"
                  >
                    {{ t("version.setAsDefault") }}
                  </button>
                  <button class="ghost" @click="openRenameModal(version)">
                    {{ t("version.rename") }}
                  </button>
                  <button
                    class="ghost danger"
                    :disabled="version.id === skillPackage?.defaultVersion"
                    @click="openDeleteModal(version)"
                  >
                    {{ t("version.delete") }}
                  </button>
                </div>
              </div>
            </div>

            <div class="variants-section">
              <h4>{{ t("version.variants") }}</h4>
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
                    <button class="ghost danger" @click="handleDeleteVariant(variant.id)">
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
            </div>
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
  padding: 8px 12px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-bg);
  color: var(--color-text);
  font-size: 14px;
  min-width: 180px;
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
  padding-top: 20px;
  border-top: 1px solid var(--color-border);
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
