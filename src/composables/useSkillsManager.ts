import { computed, onMounted, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "./useToast";
import { useIdeConfig } from "./useIdeConfig";
import { useMarketConfig } from "./useMarketConfig";
import { useMarketplaceSearch } from "./useMarketplaceSearch";
import { useDownloadQueue } from "./useDownloadQueue";
import { useLocalInventory } from "./useLocalInventory";
import { useInstallActions } from "./useInstallActions";
import { useUninstallActions } from "./useUninstallActions";
import { useIdeAdoption } from "./useIdeAdoption";
import { useProjectScan } from "./useProjectScan";
import { useVersionManagement } from "./useVersionManagement";

export function useSkillsManager() {
  const { t } = useI18n();
  const toast = useToast();

  const activeTab = ref<"local" | "market" | "ide" | "projects" | "settings">("local");
  const updatingId = ref<string | null>(null);
  const inventoryBusy = ref(false);
  const inventoryBusyText = ref("");

  const {
    marketConfigs,
    enabledMarkets,
    marketStatuses,
    loadMarketConfigs,
    saveMarketConfigs
  } = useMarketConfig();

  const {
    ideOptions,
    selectedIdeFilter,
    customIdeName,
    customIdeDir,
    customIdeOptions,
    refreshIdeOptions,
    addCustomIde: doAddCustomIde,
    removeCustomIde
  } = useIdeConfig();

  const {
    query,
    results,
    total,
    limit,
    offset,
    loading,
    hasMore,
    searchMarketplace: searchMarketplaceInternal
  } = useMarketplaceSearch(marketConfigs, enabledMarkets, marketStatuses);

  const {
    localSkills,
    ideSkills,
    localLoading,
    localSkillNameSet,
    scanLocalSkills,
    importLocalSkill: importLocalSkillInternal,
    openSkillDirectory: openSkillDirectoryInternal
  } = useLocalInventory(ideOptions, (msg) => toast.success(msg), (msg) => toast.error(msg), t);

  const {
    downloadQueue,
    recentTaskStatus,
    addToDownloadQueue,
    removeFromQueue,
    retryDownload,
    downloadSkill,
    updateSkill,
    cleanup: cleanupDownloadQueue
  } = useDownloadQueue((msg) => toast.success(msg), t, scanLocalSkills);

  const {
    showInstallModal,
    installTargetIde,
    installingId,
    busy: installBusy,
    busyText: installBusyText,
    openInstallModal,
    updateInstallTargetIde,
    closeInstallModal,
    confirmInstallToIde
  } = useInstallActions(ideOptions, (msg) => toast.success(msg), (msg) => toast.error(msg), scanLocalSkills, t);

  const {
    showUninstallModal,
    uninstallTargetName,
    uninstallMode,
    busy: uninstallBusy,
    busyText: uninstallBusyText,
    openUninstallModal,
    openUninstallManyModal,
    openDeleteLocalModal,
    confirmUninstall,
    cancelUninstall
  } = useUninstallActions(ideOptions, (msg) => toast.success(msg), (msg) => toast.error(msg), scanLocalSkills, t);

  const {
    busy: adoptionBusy,
    busyText: adoptionBusyText,
    adoptIdeSkill,
    adoptManyIdeSkills
  } = useIdeAdoption((msg) => toast.success(msg), (msg) => toast.error(msg), scanLocalSkills, t);

  const {
    projectSkillScanResult,
    showConflictModal,
    currentConflictSkill,
    busy: scanBusy,
    busyText: scanBusyText,
    scanProjectSkills,
    resolveConflict,
    openConflictModal,
    closeConflictModal
  } = useProjectScan((msg) => toast.success(msg), (msg) => toast.error(msg), t);

  const {
    currentSkillPackage,
    showVersionManagerModal,
    versionLoading,
    currentConflictAnalysis,
    showVersionDiffModal,
    currentVersionDiff,
    busy: versionBusy,
    busyText: versionBusyText,
    loadSkillPackage,
    compareVersions,
    createVersion,
    analyzeConflict,
    renameVersion,
    deleteVersion,
    setDefaultVersion,
    createVariant,
    updateVariant,
    deleteVariant,
    openVersionManagerModal,
    closeVersionManagerModal,
    openVersionDiffModal,
    closeVersionDiffModal
  } = useVersionManagement((msg) => toast.success(msg), (msg) => toast.error(msg), scanLocalSkills, t);

  const filteredIdeSkills = computed(() =>
    ideSkills.value.filter((skill) => skill.ide === selectedIdeFilter.value)
  );

  const busy = computed(() =>
    inventoryBusy.value || installBusy.value || uninstallBusy.value || adoptionBusy.value || scanBusy.value || versionBusy.value
  );

  const busyText = computed(() =>
    inventoryBusyText.value || installBusyText.value || uninstallBusyText.value || adoptionBusyText.value || scanBusyText.value || versionBusyText.value
  );

  async function searchMarketplace(reset = true, force = false): Promise<void> {
    try {
      await searchMarketplaceInternal(reset, force);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t("errors.searchFailed"));
    }
  }

  async function importLocalSkill(): Promise<void> {
    await importLocalSkillInternal((nextBusy, text) => {
      inventoryBusy.value = nextBusy;
      inventoryBusyText.value = text;
    });
  }

  async function openSkillDirectory(path: string): Promise<void> {
    await openSkillDirectoryInternal(path);
  }

  function addCustomIde() {
    const success = doAddCustomIde(t, (msg: string) => {
      toast.error(msg);
    });
    if (success) {
      void scanLocalSkills();
    }
  }

  onMounted(() => {
    refreshIdeOptions();
    loadMarketConfigs();
    void searchMarketplace(true);
    void scanLocalSkills();
  });

  onUnmounted(() => {
    cleanupDownloadQueue();
  });

  return {
    activeTab,
    query,
    results,
    total,
    limit,
    offset,
    loading,
    installingId,
    updatingId,
    localSkills,
    ideSkills,
    localLoading,
    ideOptions,
    selectedIdeFilter,
    customIdeName,
    customIdeDir,
    showInstallModal,
    installTargetIde,
    showUninstallModal,
    uninstallTargetName,
    busy,
    busyText,
    hasMore,
    localSkillNameSet,
    filteredIdeSkills,
    customIdeOptions,
    marketConfigs,
    marketStatuses,
    enabledMarkets,
    downloadQueue,
    uninstallMode,
    recentTaskStatus,
    projectSkillScanResult,
    showConflictModal,
    currentConflictSkill,
    currentSkillPackage,
    showVersionManagerModal,
    versionLoading,
    currentConflictAnalysis,
    showVersionDiffModal,
    currentVersionDiff,
    refreshIdeOptions,
    addCustomIde,
    removeCustomIde,
    saveMarketConfigs,
    searchMarketplace,
    downloadSkill,
    updateSkill,
    scanLocalSkills,
    openInstallModal,
    updateInstallTargetIde,
    confirmInstallToIde,
    closeInstallModal,
    openUninstallModal,
    openUninstallManyModal,
    openDeleteLocalModal,
    confirmUninstall,
    cancelUninstall,
    importLocalSkill,
    openSkillDirectory,
    adoptIdeSkill,
    adoptManyIdeSkills,
    addToDownloadQueue,
    removeFromQueue,
    retryDownload,
    scanProjectSkills,
    resolveConflict,
    openConflictModal,
    closeConflictModal,
    loadSkillPackage,
    compareVersions,
    createVersion,
    analyzeConflict,
    renameVersion,
    deleteVersion,
    setDefaultVersion,
    createVariant,
    updateVariant,
    deleteVariant,
    openVersionManagerModal,
    closeVersionManagerModal,
    openVersionDiffModal,
    closeVersionDiffModal
  };
}
