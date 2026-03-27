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
  display: inline-block;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
  white-space: nowrap;
}
.sync-tag--green {
  background: rgba(34, 197, 94, 0.15);
  color: #16a34a;
}
.sync-tag--orange {
  background: rgba(249, 115, 22, 0.15);
  color: #ea580c;
}
.sync-tag--blue {
  background: rgba(59, 130, 246, 0.15);
  color: #2563eb;
}
.sync-tag--red {
  background: rgba(239, 68, 68, 0.15);
  color: #dc2626;
}
.sync-tag--gray {
  background: rgba(107, 114, 128, 0.15);
  color: #6b7280;
}
</style>
