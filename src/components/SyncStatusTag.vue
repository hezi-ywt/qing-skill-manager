<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

const props = defineProps<{
  syncStatus: string;
  syncMode?: string | null;
  syncBranch?: string | null;
}>();

const { t } = useI18n();

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
</script>

<template>
  <span class="sync-tag" :class="`sync-tag--${tagConfig.color}`">
    {{ tagConfig.label }}
  </span>
</template>

<style scoped>
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
</style>
