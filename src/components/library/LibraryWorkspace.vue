<script setup lang="ts">
import { computed, ref, watch } from "vue";
import LibrarySidebar from "./LibrarySidebar.vue";
import LibraryDetailPanel from "./LibraryDetailPanel.vue";
import LibraryVersionRail from "./LibraryVersionRail.vue";
import type {
  DownloadTask,
  IdeOption,
  LibrarySkill,
  LocalSkill,
  ProjectConfig,
  SkillPackage,
  SkillVersion
} from "../../composables/types";

type CloneTargetProject = {
  id: string;
  name: string;
  ideTargets: string[];
};

const props = defineProps<{
  localSkills: LocalSkill[];
  localLoading: boolean;
  installingId: string | null;
  downloadQueue: DownloadTask[];
  ideOptions: IdeOption[];
  skillPackage: SkillPackage | null;
  versionLoading: boolean;
  projects: ProjectConfig[];
  librarySkills: LibrarySkill[];
}>();

const emit = defineEmits<{
  (e: "install", skill: LocalSkill): void;
  (e: "installMany", skills: LocalSkill[]): void;
  (e: "deleteLocal", skills: LocalSkill[]): void;
  (e: "openDir", path: string): void;
  (e: "refresh"): void;
  (e: "import"): void;
  (e: "retryDownload", taskId: string): void;
  (e: "removeFromQueue", taskId: string): void;
  (e: "manageVersions", skill: LocalSkill): void;
  (e: "setDefaultVersion", skillId: string, versionId: string): void;
  (e: "cloneToProject", project: CloneTargetProject, skillIds: string[]): void;
  (e: "compareVersions", fromVersionId: string, toVersionId: string): void;
  (e: "createVersion"): void;
}>();

const searchQuery = ref("");
const selectedSkillId = ref<string | null>(null);
const selectedIds = ref<string[]>([]);
const selectedVersionId = ref<string | null>(null);
const platformFilter = ref<string>("all");
const statusFilter = ref<string>("all");

const platformOptions = computed(() => {
  const options = [{ id: "all", label: "all", count: props.localSkills.length }];
  for (const ide of props.ideOptions) {
    options.push({
      id: ide.id,
      label: ide.label,
      count: props.librarySkills.filter((skill) => skill.installations.some((item) => item.ideId === ide.id)).length
    });
  }
  return options;
});

const statusOptions = computed(() => {
  const counts = {
    all: props.librarySkills.length,
    linked: props.librarySkills.filter((skill) => skill.usedByProjectIds.length > 0).length,
    active: props.localSkills.filter((skill) => skill.currentVersion?.isActive).length,
    unused: props.localSkills.filter((skill) => skill.usedBy.length === 0 && !skill.currentVersion?.isActive).length
  };

  return [
    { id: "all", label: "all", count: counts.all },
    { id: "linked", label: "linked", count: counts.linked },
    { id: "active", label: "active", count: counts.active },
    { id: "unused", label: "unused", count: counts.unused }
  ];
});

const filteredSidebarSkills = computed<LocalSkill[]>(() => {
  const keyword = searchQuery.value.trim().toLowerCase();

  return props.localSkills.filter((skill) => {
    const librarySkill = props.librarySkills.find((item) => item.id === skill.id);
    if (!librarySkill) {
      return false;
    }

    if (platformFilter.value !== "all" && !librarySkill.installations.some((item) => item.ideId === platformFilter.value)) {
      return false;
    }

    if (statusFilter.value !== "all") {
      if (statusFilter.value === "linked" && librarySkill.usedByProjectIds.length === 0) {
        return false;
      }
      if (statusFilter.value === "active" && !skill.currentVersion?.isActive) {
        return false;
      }
      if (statusFilter.value === "unused" && (skill.usedBy.length > 0 || skill.currentVersion?.isActive)) {
        return false;
      }
    }

    if (!keyword) {
      return true;
    }

    return [skill.name, skill.description, skill.path].some((value) => value.toLowerCase().includes(keyword));
  });
});

const selectedSkill = computed<LocalSkill | null>(() => {
  if (!selectedSkillId.value) {
    return null;
  }
  return props.localSkills.find((skill) => skill.id === selectedSkillId.value) || null;
});

const selectedLibrarySkill = computed<LibrarySkill | null>(() => {
  if (!selectedSkillId.value) {
    return null;
  }
  return props.librarySkills.find((skill) => skill.id === selectedSkillId.value) || null;
});

const selectedVersion = computed<SkillVersion | null>(() => {
  if (!selectedVersionId.value || !props.skillPackage) {
    return null;
  }
  return props.skillPackage.versions.find((version) => version.id === selectedVersionId.value) || null;
});

const selectedSkills = computed<LocalSkill[]>(() =>
  props.localSkills.filter((skill) => selectedIds.value.includes(skill.id))
);

watch(
  () => props.localSkills,
  (skills) => {
    const availableIds = new Set(skills.map((skill) => skill.id));
    selectedIds.value = selectedIds.value.filter((id) => availableIds.has(id));

    if (selectedSkillId.value && !availableIds.has(selectedSkillId.value)) {
      selectedSkillId.value = null;
    }

    if (skills.length > 0 && !selectedSkillId.value) {
      selectedSkillId.value = skills[0].id;
    }
  },
  { immediate: true }
);

watch(selectedSkillId, (skillId) => {
  if (!skillId) {
    selectedVersionId.value = null;
    return;
  }

  const currentVersion = props.skillPackage?.versions.find((version) => version.isActive)
    || props.skillPackage?.versions.find((version) => version.id === props.skillPackage?.defaultVersion)
    || null;
  selectedVersionId.value = currentVersion?.id || null;
});

watch(
  () => props.skillPackage,
  (skillPackage) => {
    if (!skillPackage) {
      selectedVersionId.value = null;
      return;
    }

    if (selectedVersionId.value && skillPackage.versions.some((version) => version.id === selectedVersionId.value)) {
      return;
    }

    const nextVersion = skillPackage.versions.find((version) => version.isActive)
      || skillPackage.versions.find((version) => version.id === skillPackage.defaultVersion)
      || skillPackage.versions[0]
      || null;
    selectedVersionId.value = nextVersion?.id || null;
  },
  { immediate: true }
);

function handleSelectSkill(skill: LocalSkill): void {
  selectedSkillId.value = skill.id;
}

function handleToggleSelected(skillId: string, checked: boolean): void {
  selectedIds.value = checked
    ? Array.from(new Set([...selectedIds.value, skillId]))
    : selectedIds.value.filter((id) => id !== skillId);
}

function handleToggleSelectAll(checked: boolean, filteredIds: string[]): void {
  if (checked) {
    selectedIds.value = Array.from(new Set([...selectedIds.value, ...filteredIds]));
    return;
  }

  selectedIds.value = selectedIds.value.filter((id) => !filteredIds.includes(id));
}

watch(filteredSidebarSkills, (skills) => {
  const visibleIds = new Set(skills.map((skill) => skill.id));
  selectedIds.value = selectedIds.value.filter((id) => visibleIds.has(id) || props.localSkills.some((skill) => skill.id === id));

  if (selectedSkillId.value && !visibleIds.has(selectedSkillId.value)) {
    selectedSkillId.value = skills[0]?.id || null;
  }
});

function handleDelete(skill: LocalSkill): void {
  emit("deleteLocal", [skill]);
  if (selectedSkillId.value === skill.id) {
    selectedSkillId.value = null;
  }
}

function handleDeleteSelected(): void {
  if (selectedSkills.value.length === 0) {
    return;
  }

  emit("deleteLocal", selectedSkills.value);
}

function handleClearSelection(): void {
  selectedIds.value = [];
}

function handleInstallSelected(): void {
  if (selectedSkills.value.length === 0) {
    return;
  }

  emit("installMany", selectedSkills.value);
}

function handleDeleteAll(): void {
  if (props.localSkills.length === 0) {
    return;
  }

  emit("deleteLocal", props.localSkills);
}

function handleSetDefaultVersion(versionId: string): void {
  if (!props.skillPackage) {
    return;
  }
  emit("setDefaultVersion", props.skillPackage.id, versionId);
}

function handleCloneToProject(projectId: string): void {
  const project = props.projects.find((item) => item.id === projectId);
  if (!project || !selectedSkill.value) {
    return;
  }

  emit("cloneToProject", {
    id: project.id,
    name: project.name,
    ideTargets: project.ideTargets
  }, [selectedSkill.value.id]);
}

function handleCompareSelectedVersion(versionId: string): void {
  if (!selectedSkill.value?.currentVersion || selectedSkill.value.currentVersion.id === versionId) {
    return;
  }

  emit("compareVersions", selectedSkill.value.currentVersion.id, versionId);
}

function handleSelectVersion(version: SkillVersion): void {
  selectedVersionId.value = version.id;
}
</script>

<template>
  <div class="library-workspace">
    <LibrarySidebar
      v-model:search-query="searchQuery"
      :skills="filteredSidebarSkills"
      :selected-skill-id="selectedSkillId"
      :selected-ids="selectedIds"
      :loading="localLoading"
      :ide-options="ideOptions"
      :platform-filter="platformFilter"
      :status-filter="statusFilter"
      :platform-options="platformOptions"
      :status-options="statusOptions"
      @select="handleSelectSkill"
      @toggle-selected="handleToggleSelected"
      @toggle-select-all="handleToggleSelectAll"
      @install-selected="handleInstallSelected"
      @delete-selected="handleDeleteSelected"
      @clear-selection="handleClearSelection"
      @delete-all="handleDeleteAll"
      @update:platform-filter="platformFilter = $event"
      @update:status-filter="statusFilter = $event"
      @refresh="$emit('refresh')"
      @import="$emit('import')"
    />

    <LibraryDetailPanel
      :skill="selectedSkill"
      :library-skill="selectedLibrarySkill"
      :installing-id="installingId"
      :ide-options="ideOptions"
      :projects="projects"
      @install="$emit('install', $event)"
      @clone-to-project="handleCloneToProject"
      @open-dir="$emit('openDir', $event)"
      @manage-versions="$emit('manageVersions', $event)"
      @delete="handleDelete"
    />

    <LibraryVersionRail
      :skill="selectedSkill"
      :selected-version-id="selectedVersion?.id || null"
      :skill-package="skillPackage"
      :loading="versionLoading"
      @select-version="handleSelectVersion"
      @compare-versions="handleCompareSelectedVersion"
      @create-version="$emit('createVersion')"
      @set-default="handleSetDefaultVersion"
    />
  </div>
</template>

<style scoped>
.library-workspace {
  display: flex;
  height: 100%;
  width: 100%;
  overflow: hidden;
  background: var(--color-bg);
}
</style>
