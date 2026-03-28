<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";

const props = defineProps<{
  syncStatus: string;
  syncMode?: string | null;
  syncBranch?: string | null;
  editable?: boolean;
}>();

const emit = defineEmits<{
  (e: "update", syncMode: string, syncBranch: string): void;
}>();

const { t } = useI18n();

const showPopover = ref(false);
const editMode = ref<"sync" | "independent">("sync");
const editBranch = ref("main");
const customBranch = ref("");
const builtinBranches = ["main", "dev", "stable"];

const tagConfig = computed(() => {
  if (props.syncMode === "independent") {
    return { label: t("sync.independent"), color: "gray" };
  }

  const branch = props.syncBranch || "main";
  switch (props.syncStatus) {
    case "synced":
      return { label: `${t("sync.synced")} · ${branch}`, color: "green" };
    case "outdated":
      return { label: `${t("sync.outdated")} · ${branch}`, color: "orange" };
    case "modified":
    case "diverged":
      return { label: `${t("sync.diverged")} · ${branch}`, color: "blue" };
    case "conflict":
      return { label: `${t("sync.conflict")} · ${branch}`, color: "red" };
    case "untracked":
      return { label: t("sync.untracked"), color: "gray" };
    default:
      return { label: t("sync.unknown"), color: "gray" };
  }
});

function openPopover() {
  if (!props.editable) return;
  editMode.value = (props.syncMode as "sync" | "independent") || "sync";
  const branch = props.syncBranch || "main";
  if (builtinBranches.includes(branch)) {
    editBranch.value = branch;
    customBranch.value = "";
  } else {
    editBranch.value = "__custom__";
    customBranch.value = branch;
  }
  showPopover.value = true;
}

function confirmEdit() {
  const branch = editBranch.value === "__custom__" ? customBranch.value : editBranch.value;
  emit("update", editMode.value, branch);
  showPopover.value = false;
}

function closePopover() {
  showPopover.value = false;
}
</script>

<template>
  <span class="sync-tag-wrapper">
    <span
      class="sync-tag"
      :class="[`sync-tag--${tagConfig.color}`, { clickable: editable }]"
      @click="openPopover"
    >
      {{ tagConfig.label }}
    </span>

    <!-- Edit popover -->
    <Teleport to="body">
      <div v-if="showPopover" class="popover-overlay" @click.self="closePopover">
        <div class="sync-popover">
          <div class="popover-title">{{ t("sync.editSettings") }}</div>

          <div class="popover-section">
            <div class="popover-label">{{ t("installModal.syncBranch") }}</div>
            <div class="popover-chips">
              <button
                v-for="b in builtinBranches"
                :key="b"
                class="branch-chip"
                :class="{ active: editBranch === b && editMode === 'sync' }"
                :disabled="editMode === 'independent'"
                @click="editBranch = b"
              >{{ b }}</button>
              <button
                class="branch-chip"
                :class="{ active: editBranch === '__custom__' && editMode === 'sync' }"
                :disabled="editMode === 'independent'"
                @click="editBranch = '__custom__'"
              >{{ t("installModal.customBranch") }}</button>
            </div>
            <input
              v-if="editBranch === '__custom__' && editMode === 'sync'"
              v-model="customBranch"
              class="popover-input"
              :placeholder="t('installModal.customBranchPlaceholder')"
              @keydown.enter="confirmEdit"
            />
          </div>

          <div class="popover-section">
            <div class="popover-label">{{ t("installModal.syncOptions") }}</div>
            <div class="popover-mode-row">
              <label class="mode-option" :class="{ active: editMode === 'sync' }">
                <input type="radio" v-model="editMode" value="sync" />
                {{ t("installModal.syncMode") }}
              </label>
              <label class="mode-option" :class="{ active: editMode === 'independent' }">
                <input type="radio" v-model="editMode" value="independent" />
                {{ t("installModal.independentMode") }}
              </label>
            </div>
          </div>

          <div class="popover-actions">
            <button class="popover-btn primary" @click="confirmEdit">{{ t("sync.confirm") }}</button>
            <button class="popover-btn ghost" @click="closePopover">{{ t("sync.cancel") }}</button>
          </div>
        </div>
      </div>
    </Teleport>
  </span>
</template>

<style scoped>
.sync-tag-wrapper {
  position: relative;
  display: inline-flex;
}

.sync-tag {
  display: inline-flex;
  align-items: center;
  padding: 3px 8px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  white-space: nowrap;
  line-height: 1.3;
}

.sync-tag.clickable {
  cursor: pointer;
  transition: opacity 0.15s;
}

.sync-tag.clickable:hover {
  opacity: 0.8;
}

.sync-tag--green {
  background: var(--color-success-bg);
  border: 1px solid var(--color-success-border);
  color: var(--color-success-text);
}
.sync-tag--orange {
  background: var(--color-chip-bg);
  border: 1px solid var(--color-chip-border);
  color: var(--color-text);
}
.sync-tag--blue {
  background: var(--color-chip-bg);
  border: 1px solid var(--color-chip-border);
  color: var(--color-text);
}
.sync-tag--red {
  background: var(--color-error-bg);
  border: 1px solid var(--color-error-border);
  color: var(--color-error-text);
}
.sync-tag--gray {
  background: var(--color-card-bg);
  border: 1px solid var(--color-card-border);
  color: var(--color-muted);
}

/* Popover */
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
  padding: 16px;
  width: 300px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
}

.popover-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: 14px;
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

.popover-btn {
  padding: 5px 14px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  border: none;
}

.popover-btn.primary {
  background: var(--color-success-text, #16a34a);
  color: #fff;
}

.popover-btn.primary:hover {
  opacity: 0.9;
}

.popover-btn.ghost {
  background: transparent;
  color: var(--color-muted);
  border: 1px solid var(--color-card-border);
}

.popover-btn.ghost:hover {
  background: var(--color-card-bg);
}
</style>
