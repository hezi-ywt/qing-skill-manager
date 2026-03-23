import { computed, onMounted, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { dirname, homeDir, join } from "@tauri-apps/api/path";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { useToast } from "./useToast";
import type {
  RemoteSkill, MarketStatus, InstallResult, LocalSkill,
  IdeSkill, Overview, LinkTarget, DownloadTask, ProjectConfig,
  ProjectSkill, ProjectSkillScanResult, ConflictResolution, ResolveConflictResult,
  SkillPackage, SkillDiff, ConflictAnalysis,
  DeleteVersionRequest, DeleteVersionResponse, DeleteStrategy, AnalyzeConflictRequest,
  GetSkillPackageResponse, RenameVersionResponse, CompareVersionsRequest,
  CreateVariantRequest, CreateVariantResponse, UpdateVariantRequest, DeleteVariantRequest, SkillVariant,
  CreateVersionRequest, CreateVersionResponse
} from "./types";
import { useIdeConfig } from "./useIdeConfig";
import { useMarketConfig } from "./useMarketConfig";
import { isSafeRelativePath, getErrorMessage, isSafeAbsolutePath } from "./utils";

export function useSkillsManager() {
  const { t } = useI18n();
  const toast = useToast();
  const cacheTtlMs = 10 * 60 * 1000;
  const searchCache = new Map<
    string,
    { timestamp: number; data: { skills: RemoteSkill[]; total: number; limit: number; offset: number; marketStatuses: MarketStatus[] } }
  >();
  const activeTab = ref<"local" | "market" | "ide" | "projects" | "settings">("local");

  const query = ref("");
  const results = ref<RemoteSkill[]>([]);
  const total = ref(0);
  const limit = ref(20);
  const offset = ref(0);
  const loading = ref(false);
  const installingId = ref<string | null>(null);
  const updatingId = ref<string | null>(null);

  // Local Skills
  const localSkills = ref<LocalSkill[]>([]);
  const ideSkills = ref<IdeSkill[]>([]);
  const localLoading = ref(false);

  // Download Queue
  const downloadQueue = ref<DownloadTask[]>([]);
  let isProcessingQueue = false;

  // Timer tracking for cleanup
  const timers: number[] = [];

  // Cleanup on unmount
  onUnmounted(() => {
    timers.forEach((id) => clearTimeout(id));
  });

  const showInstallModal = ref(false);
  const installTargetSkills = ref<LocalSkill[]>([]);
  const installTargetIde = ref<string[]>([]);

  const showUninstallModal = ref(false);
  const uninstallTargetPath = ref("");
  const uninstallTargetName = ref("");
  const uninstallTargetPaths = ref<string[]>([]);
  const uninstallMode = ref<"ide" | "local">("ide");

  const busy = ref(false);
  const busyText = ref("");
  const recentTaskStatus = ref<Record<string, "download" | "update">>({});

  const hasMore = computed(() => results.value.length < total.value);
  const localSkillNameSet = computed(() => {
    const set = new Set<string>();
    for (const skill of localSkills.value) {
      const key = skill.name.trim().toLowerCase();
      if (key) set.add(key);
    }
    return set;
  });

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
    removeCustomIde,
    loadLastInstallTargets,
    saveLastInstallTargets
  } = useIdeConfig();

  function addCustomIde() {
    const success = doAddCustomIde(t, (msg: string) => {
      toast.error(msg);
    });
    if (success) {
      void scanLocalSkills();
    }
  }

  const filteredIdeSkills = computed(() =>
    ideSkills.value.filter((skill) => skill.ide === selectedIdeFilter.value)
  );
  async function buildInstallBaseDir(): Promise<string> {
    const home = await homeDir();
    return join(home, ".skills-manager/skills");
  }

  async function buildLinkTargets(targetLabel: string): Promise<LinkTarget[]> {
    const target = ideOptions.value.find((option) => option.label === targetLabel);
    if (!target) return [];

    const dir = target.globalDir;

    // Absolute path: use directly
    if (isSafeAbsolutePath(dir)) {
      return [{ name: target.label, path: dir }];
    }

    // Relative path: join with home directory
    if (!isSafeRelativePath(dir)) return [];

    const home = await homeDir();
    return [
      {
        name: target.label,
        path: await join(home, dir)
      }
    ];
  }

  async function searchMarketplace(reset = true, force = false) {
    if (loading.value) return;
    loading.value = true;

    const nextOffset = reset ? 0 : offset.value + limit.value;
    const cacheKey = `${query.value.trim().toLowerCase()}|${limit.value}`;

    if (reset && !force) {
      const cached = searchCache.get(cacheKey);
      if (cached && Date.now() - cached.timestamp < cacheTtlMs) {
        results.value = cached.data.skills;
        total.value = cached.data.total;
        offset.value = cached.data.offset;
        marketStatuses.value = cached.data.marketStatuses;
        loading.value = false;
        return;
      }
    }

    try {
      const response = await invoke("search_marketplaces", {
        query: query.value,
        limit: limit.value,
        offset: nextOffset,
        apiKeys: marketConfigs.value,
        enabledMarkets: enabledMarkets.value
      });
      const data = response as {
        skills: RemoteSkill[];
        total: number;
        limit: number;
        offset: number;
        marketStatuses: MarketStatus[];
      };

      const deduped = dedupeSkills(reset ? data.skills : [...results.value, ...data.skills]);
      results.value = deduped;

      total.value = data.total;
      offset.value = data.offset;
      if (Array.isArray(data.marketStatuses)) {
        marketStatuses.value = data.marketStatuses;
      }

      if (reset) {
        const cachedStatuses = Array.isArray(data.marketStatuses)
          ? data.marketStatuses
          : marketStatuses.value;
        searchCache.set(cacheKey, {
          timestamp: Date.now(),
          data: { ...data, marketStatuses: cachedStatuses }
        });
      }
    } catch (err) {
      toast.error(getErrorMessage(err, t("errors.searchFailed")));
    } finally {
      loading.value = false;
    }
  }

  function dedupeSkills(skills: RemoteSkill[]) {
    const map = new Map<string, RemoteSkill>();
    for (const skill of skills) {
      const sourceKey = skill.sourceUrl?.trim().toLowerCase();
      const nameKey = `${skill.marketId}:${skill.name.trim().toLowerCase()}`;
      const key = sourceKey || nameKey;
      if (!map.has(key)) {
        map.set(key, skill);
      }
    }
    return Array.from(map.values());
  }

  function addToDownloadQueue(skill: RemoteSkill, action: "download" | "update" = "download") {
    // Check if already in queue
    if (downloadQueue.value.some(t => t.id === skill.id)) {
      return;
    }
    downloadQueue.value.push({
      id: skill.id,
      name: skill.name,
      sourceUrl: skill.sourceUrl,
      action,
      status: 'pending'
    });
    processQueue();
  }

  async function processQueue() {
    if (isProcessingQueue) return;
    isProcessingQueue = true;

    while (true) {
      const task = downloadQueue.value.find(t => t.status === 'pending');
      if (!task) break;

      task.status = 'downloading';
      try {
        const installBaseDir = await buildInstallBaseDir();
        const command = task.action === "update"
          ? "update_marketplace_skill"
          : "download_marketplace_skill";

        await invoke(command, {
          request: {
            sourceUrl: task.sourceUrl,
            skillName: task.name,
            installBaseDir
          }
        });
        task.status = 'done';
        recentTaskStatus.value = {
          ...recentTaskStatus.value,
          [task.id]: task.action
        };
        toast.success(
          task.action === "update"
            ? t("messages.updated", { path: task.name })
            : t("messages.downloaded", { path: task.name })
        );
        // Remove completed task after a short delay
        const timerId = window.setTimeout(() => {
          downloadQueue.value = downloadQueue.value.filter(t => t.id !== task.id);
          const nextStatus = { ...recentTaskStatus.value };
          delete nextStatus[task.id];
          recentTaskStatus.value = nextStatus;
          void scanLocalSkills(); // Properly handle async
          // Clean up timer to prevent memory leaks
          const index = timers.indexOf(timerId);
          if (index > -1) timers.splice(index, 1);
        }, 2500);
        timers.push(timerId);
      } catch (err) {
        task.status = 'error';
        task.error = err instanceof Error ? err.message : String(err);
      }
    }

    isProcessingQueue = false;
  }

  function removeFromQueue(taskId: string) {
    downloadQueue.value = downloadQueue.value.filter(t => t.id !== taskId);
  }

  function retryDownload(taskId: string) {
    const task = downloadQueue.value.find(t => t.id === taskId);
    if (task && task.status === 'error') {
      task.status = 'pending';
      task.error = undefined;
      processQueue();
    }
  }

  // Keep original downloadSkill for backward compatibility
  async function downloadSkill(skill: RemoteSkill) {
    addToDownloadQueue(skill, "download");
  }

  async function updateSkill(skill: RemoteSkill) {
    addToDownloadQueue(skill, "update");
  }

  async function scanLocalSkills() {
    if (localLoading.value) return;
    localLoading.value = true;

    try {
      const response = (await invoke("scan_overview", {
        request: {
          projectDir: null,
          ideDirs: ideOptions.value.map((item) => ({
            label: item.label,
            relativeDir: item.globalDir
          }))
        }
      })) as Overview;
      localSkills.value = response.managerSkills;
      ideSkills.value = response.ideSkills;
    } catch (err) {
      toast.error(getErrorMessage(err, t("errors.scanFailed")));
    } finally {
      localLoading.value = false;
    }
  }

  async function linkSkillInternal(skill: LocalSkill, ideLabel: string, skipScan = false, suppressToast = false) {
    const linkTargets = await buildLinkTargets(ideLabel);
    if (linkTargets.length === 0) {
      throw new Error(t("errors.selectValidIde"));
    }
    const result = (await invoke("link_local_skill", {
      request: {
        skillPath: skill.path,
        skillName: skill.name,
        linkTargets
      }
    })) as InstallResult;

    const linkedCount = result.linked.length;
    const skippedCount = result.skipped.length;
    if (!suppressToast) {
      toast.success(t("messages.handled", { linked: linkedCount, skipped: skippedCount }));
    }
    if (!skipScan) {
      await scanLocalSkills();
    }
    return result;
  }

  function openInstallModal(skill: LocalSkill | LocalSkill[]) {
    installTargetSkills.value = Array.isArray(skill) ? skill : [skill];
    const lastTargets = loadLastInstallTargets();
    const available = new Set(ideOptions.value.map((item) => item.label));
    const nextTargets = lastTargets.filter((label) => available.has(label));
    installTargetIde.value = nextTargets;
    showInstallModal.value = true;
  }

  function updateInstallTargetIde(next: string[]) {
    installTargetIde.value = next;
    saveLastInstallTargets(next);
  }

  async function confirmInstallToIde(installTarget: "ide" | "project", targetIds: string[], projects?: ProjectConfig[]) {
    if (installTarget === "project") {
      // Project installation
      if (!projects || projects.length === 0) {
        toast.error("No projects available");
        showInstallModal.value = false;
        installTargetSkills.value = [];
        return;
      }
      
      if (installTargetSkills.value.length === 0 || targetIds.length === 0) {
        toast.error(t("errors.selectAtLeastOne"));
        return;
      }
      if (installingId.value) return;
      installingId.value = installTargetSkills.value.length === 1 ? installTargetSkills.value[0].id : "__batch__";
      busy.value = true;
      busyText.value = t("messages.installing");

      try {
        let totalLinked = 0;
        let totalSkipped = 0;
        
        // Get selected projects
        const selectedProjects = projects.filter(p => targetIds.includes(p.id));
        
        // Install to project directories
        for (const skill of installTargetSkills.value) {
          for (const project of selectedProjects) {
            for (const ideLabel of project.ideTargets) {
              const result = await linkSkillToProjectInternal(skill, project.path, ideLabel, true, true);
              totalLinked += result.linked.length;
              totalSkipped += result.skipped.length;
            }
          }
        }
        
        toast.success(t("messages.handled", { linked: totalLinked, skipped: totalSkipped }));
        await scanLocalSkills();
        showInstallModal.value = false;
        installTargetSkills.value = [];
      } catch (err) {
        toast.error(getErrorMessage(err, t("errors.installFailed")));
      } finally {
        installingId.value = null;
        busy.value = false;
        busyText.value = "";
      }
      return;
    }
    
    // IDE installation (existing logic)
    if (installTargetSkills.value.length === 0 || targetIds.length === 0) {
      toast.error(t("errors.selectAtLeastOne"));
      return;
    }
    if (installingId.value) return;
    installingId.value = installTargetSkills.value.length === 1 ? installTargetSkills.value[0].id : "__batch__";
    busy.value = true;
    busyText.value = t("messages.installing");

    try {
      let totalLinked = 0;
      let totalSkipped = 0;
      
      // Install to global IDE directories
      for (const skill of installTargetSkills.value) {
        for (const label of targetIds) {
          const result = await linkSkillInternal(skill, label, true, true);
          totalLinked += result.linked.length;
          totalSkipped += result.skipped.length;
        }
      }
      
      toast.success(t("messages.handled", { linked: totalLinked, skipped: totalSkipped }));
      await scanLocalSkills();
      showInstallModal.value = false;
      installTargetSkills.value = [];
    } catch (err) {
      toast.error(getErrorMessage(err, t("errors.installFailed")));
    } finally {
      installingId.value = null;
      busy.value = false;
      busyText.value = "";
    }
  }

  async function linkSkillToProjectInternal(skill: LocalSkill, projectPath: string, ideLabel: string, skipScan = false, suppressToast = false) {
    const linkTargets = await buildProjectLinkTargets(projectPath, ideLabel);
    if (linkTargets.length === 0) {
      throw new Error(t("errors.selectValidIde"));
    }
    const result = (await invoke("link_local_skill", {
      request: {
        skillPath: skill.path,
        skillName: skill.name,
        linkTargets
      }
    })) as InstallResult;

    const linkedCount = result.linked.length;
    const skippedCount = result.skipped.length;
    if (!suppressToast) {
      toast.success(t("messages.handled", { linked: linkedCount, skipped: skippedCount }));
    }
    if (!skipScan) {
      await scanLocalSkills();
    }
    return result;
  }

  async function buildProjectLinkTargets(projectPath: string, ideLabel: string): Promise<LinkTarget[]> {
    const target = ideOptions.value.find((option) => option.label === ideLabel);
    if (!target) return [];

    const dir = target.globalDir;

    // Build project-specific path
    const skillPath = dir.startsWith("/") ? dir : `${projectPath}/${dir}`;
    
    return [{ name: `${target.label} (${projectPath.split('/').pop()})`, path: skillPath }];
  }

  function closeInstallModal() {
    showInstallModal.value = false;
    installTargetSkills.value = [];
  }

  function openUninstallModal(targetPath: string) {
    uninstallMode.value = "ide";
    uninstallTargetPath.value = targetPath;
    uninstallTargetPaths.value = [targetPath];
    uninstallTargetName.value = targetPath.split(/[\\/]/).pop() || targetPath;
    showUninstallModal.value = true;
  }

  function openUninstallManyModal(paths: string[]) {
    if (paths.length === 0) return;
    uninstallMode.value = "ide";
    uninstallTargetPath.value = "";
    uninstallTargetPaths.value = paths;
    uninstallTargetName.value = t("ide.uninstallSelectedCount", { count: paths.length });
    showUninstallModal.value = true;
  }

  function openDeleteLocalModal(targets: LocalSkill[]) {
    uninstallMode.value = "local";
    uninstallTargetPath.value = "";
    uninstallTargetPaths.value = targets.map((skill) => skill.path);
    uninstallTargetName.value =
      targets.length === 1 ? targets[0].name : t("local.deleteSelectedCount", { count: targets.length });
    showUninstallModal.value = true;
  }

  async function confirmUninstall() {
    busy.value = true;
    busyText.value = uninstallMode.value === "local" ? t("messages.deleting") : t("messages.uninstalling");
    try {
      if (uninstallMode.value === "local") {
        const message = ((await invoke("delete_local_skills", {
          request: {
            targetPaths: uninstallTargetPaths.value
          }
        })) as string);
        toast.success(message);
      } else {
        // IDE mode: uninstall each path
        let successCount = 0;
        let failCount = 0;
        for (const targetPath of uninstallTargetPaths.value) {
          try {
            await invoke("uninstall_skill", {
              request: {
                targetPath,
                projectDir: null,
                ideDirs: ideOptions.value.map((item) => ({
                  label: item.label,
                  relativeDir: item.globalDir
                }))
              }
            });
            successCount++;
          } catch {
            failCount++;
          }
        }
        if (successCount > 0 && failCount === 0) {
          toast.success(t("messages.uninstalledCount", { count: successCount }));
        } else if (successCount > 0 && failCount > 0) {
          toast.success(t("messages.uninstalledPartial", { success: successCount, failed: failCount }));
        } else {
          toast.error(t("errors.uninstallFailed"));
        }
      }
      await scanLocalSkills();
    } catch (err) {
      toast.error(
        getErrorMessage(
          err,
          uninstallMode.value === "local" ? t("errors.deleteFailed") : t("errors.uninstallFailed")
        )
      );
    } finally {
      showUninstallModal.value = false;
      uninstallTargetPath.value = "";
      uninstallTargetName.value = "";
      uninstallTargetPaths.value = [];
      busy.value = false;
      busyText.value = "";
    }
  }

  function cancelUninstall() {
    showUninstallModal.value = false;
    uninstallTargetPath.value = "";
    uninstallTargetName.value = "";
    uninstallTargetPaths.value = [];
  }

  async function importLocalSkill() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        directory: true,
        multiple: true,
        title: t("local.selectSkillDir")
      });

      if (!selected) return;

      const paths = Array.isArray(selected) ? selected : [selected];
      if (paths.length === 0) return;

      busy.value = true;
      busyText.value = t("messages.importing");

      let successCount = 0;
      let failCount = 0;
      let lastError = "";

      for (const path of paths) {
        try {
          await invoke("import_local_skill", {
            request: {
              sourcePath: path
            }
          });
          successCount++;
        } catch (err) {
          failCount++;
          lastError = err instanceof Error ? err.message : String(err);
        }
      }

      if (successCount > 0) {
        toast.success(t("messages.imported", { success: successCount, failed: failCount }));
      } else {
        toast.error(
          t("messages.imported", { success: 0, failed: failCount }) +
          (paths.length === 1 ? `: ${lastError}` : "")
        );
      }

      await scanLocalSkills();
    } catch (err) {
      toast.error(getErrorMessage(err, t("errors.importFailed")));
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  async function openSkillDirectory(path: string) {
    try {
      await revealItemInDir(path);
    } catch (err) {
      const message = getErrorMessage(err, t("errors.openDirFailed"));
      if (message.includes("os error 2") || message.toLowerCase().includes("cannot find the file")) {
        try {
          await revealItemInDir(await dirname(path));
          toast.error(t("errors.openDirFailed") + ": " + path);
          return;
        } catch {
          // Fall through to the original error below.
        }
      }
      toast.error(message);
    }
  }

  async function adoptIdeSkill(skill: IdeSkill) {
    busy.value = true;
    busyText.value = t("messages.adopting");
    try {
      const message = (await invoke("adopt_ide_skill", {
        request: {
          targetPath: skill.path,
          ideLabel: skill.ide
        }
      })) as string;
      toast.success(message);
      await scanLocalSkills();
    } catch (err) {
      toast.error(getErrorMessage(err, t("errors.adoptFailed")));
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  async function adoptManyIdeSkills(skills: IdeSkill[]) {
    if (skills.length === 0) return;
    busy.value = true;
    busyText.value = t("messages.adopting");
    let successCount = 0;
    let failCount = 0;
    try {
      for (const skill of skills) {
        try {
          await invoke("adopt_ide_skill", {
            request: {
              targetPath: skill.path,
              ideLabel: skill.ide
            }
          });
          successCount++;
        } catch {
          failCount++;
        }
      }
      if (successCount > 0 && failCount === 0) {
        toast.success(t("messages.adoptedCount", { count: successCount }));
      } else if (successCount > 0 && failCount > 0) {
        toast.success(t("messages.adoptedPartial", { success: successCount, failed: failCount }));
      } else {
        toast.error(t("errors.adoptFailed"));
      }
      await scanLocalSkills();
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  const projectSkillScanResult = ref<ProjectSkillScanResult | null>(null);
  const showConflictModal = ref(false);
  const currentConflictSkill = ref<ProjectSkill | null>(null);

  async function scanProjectSkills(projectPath: string): Promise<ProjectSkillScanResult | null> {
    busy.value = true;
    busyText.value = t("messages.scanningProject");
    try {
      const installBaseDir = await buildInstallBaseDir();
      const result = await invoke("scan_project_opencode_skills", {
        request: {
          projectDir: projectPath,
          managerRoot: installBaseDir
        }
      }) as ProjectSkillScanResult;
      projectSkillScanResult.value = result;
      return result;
    } catch (err) {
      toast.error(getErrorMessage(err, t("errors.scanProjectFailed")));
      return null;
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  async function resolveConflict(skill: ProjectSkill, resolution: ConflictResolution, coexistName?: string): Promise<boolean> {
    busy.value = true;
    busyText.value = t("messages.resolvingConflict");
    try {
      const result = await invoke("resolve_skill_conflict", {
        request: {
          projectSkillPath: skill.path,
          resolution,
          coexistName
        }
      }) as ResolveConflictResult;

      if (result.success) {
        toast.success(t("messages.conflictResolved", { action: result.action }));
        if (projectSkillScanResult.value) {
          const skillIndex = projectSkillScanResult.value.skills.findIndex(s => s.path === skill.path);
          if (skillIndex !== -1) {
            projectSkillScanResult.value.skills[skillIndex].status = "duplicate";
          }
        }
        return true;
      } else {
        toast.error(t("errors.resolveConflictFailed"));
        return false;
      }
    } catch (err) {
      toast.error(getErrorMessage(err, t("errors.resolveConflictFailed")));
      return false;
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  function openConflictModal(skill: ProjectSkill) {
    currentConflictSkill.value = skill;
    showConflictModal.value = true;
  }

  function closeConflictModal() {
    showConflictModal.value = false;
    currentConflictSkill.value = null;
  }

  const currentSkillPackage = ref<SkillPackage | null>(null);
  const showVersionManagerModal = ref(false);
  const versionLoading = ref(false);
  const currentConflictAnalysis = ref<ConflictAnalysis | null>(null);
  const showVersionDiffModal = ref(false);
  const currentVersionDiff = ref<SkillDiff | null>(null);

  async function loadSkillPackage(skillId: string): Promise<SkillPackage | null> {
    versionLoading.value = true;
    try {
      const response = await invoke("get_skill_package", {
        request: { skillId }
      }) as GetSkillPackageResponse;
      currentSkillPackage.value = response.package;
      return response.package;
    } catch (err) {
      toast.error(getErrorMessage(err, t("errors.loadPackageFailed")));
      return null;
    } finally {
      versionLoading.value = false;
    }
  }

  async function compareVersions(skillId: string, fromVersion: string, toVersion: string): Promise<SkillDiff | null> {
    try {
      const response = await invoke("compare_skill_versions", {
        request: { skillId, fromVersion, toVersion } as CompareVersionsRequest
      }) as SkillDiff;
      currentVersionDiff.value = response;
      return response;
    } catch (err) {
      toast.error(getErrorMessage(err, t("errors.compareVersionsFailed")));
      return null;
    }
  }

  async function createVersion(request: CreateVersionRequest): Promise<CreateVersionResponse | null> {
    busy.value = true;
    busyText.value = t("messages.creatingVersion");
    try {
      const response = await invoke("create_skill_version", { request }) as CreateVersionResponse;
      if (currentSkillPackage.value?.id === request.skillId) {
        await loadSkillPackage(request.skillId);
      }
      await scanLocalSkills();
      toast.success(t("messages.versionCreated", { version: response.version.displayName }));
      return response;
    } catch (err) {
      toast.error(getErrorMessage(err, t("errors.createVersionFailed")));
      return null;
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  async function analyzeConflict(request: AnalyzeConflictRequest): Promise<ConflictAnalysis | null> {
    try {
      const response = await invoke("analyze_skill_conflict", { request }) as ConflictAnalysis;
      currentConflictAnalysis.value = response;
      return response;
    } catch (err) {
      toast.error(getErrorMessage(err, t("errors.analyzeConflictFailed")));
      return null;
    }
  }

  async function renameVersion(skillId: string, versionId: string, newDisplayName: string): Promise<boolean> {
    busy.value = true;
    busyText.value = t("messages.renamingVersion");
    try {
      const response = await invoke("rename_skill_version", {
        request: { skillId, versionId, newDisplayName }
      }) as RenameVersionResponse;
      toast.success(t("messages.versionRenamed", { name: response.version.displayName }));
      if (currentSkillPackage.value?.id === skillId) {
        await loadSkillPackage(skillId);
      }
      await scanLocalSkills();
      return true;
    } catch (err) {
      toast.error(getErrorMessage(err, t("errors.renameVersionFailed")));
      return false;
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  async function deleteVersion(skillId: string, versionId: string, strategy: DeleteStrategy, force = false): Promise<boolean> {
    busy.value = true;
    busyText.value = t("messages.deletingVersion");
    try {
      const response = await invoke("delete_skill_version", {
        request: { skillId, versionId, strategy, force }
      } as { request: DeleteVersionRequest }) as DeleteVersionResponse;
      if (response.success) {
        toast.success(t("messages.versionDeleted"));
        if (currentSkillPackage.value?.id === skillId) {
          await loadSkillPackage(skillId);
        }
        await scanLocalSkills();
        return true;
      } else {
        toast.error(response.message);
        return false;
      }
    } catch (err) {
      toast.error(getErrorMessage(err, t("errors.deleteVersionFailed")));
      return false;
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  async function setDefaultVersion(skillId: string, versionId: string): Promise<boolean> {
    busy.value = true;
    busyText.value = t("messages.settingDefaultVersion");
    try {
      await invoke("set_default_skill_version", {
        request: { skillId, versionId }
      });
      toast.success(t("messages.defaultVersionSet"));
      if (currentSkillPackage.value?.id === skillId) {
        await loadSkillPackage(skillId);
      }
      await scanLocalSkills();
      return true;
    } catch (err) {
      toast.error(getErrorMessage(err, t("errors.setDefaultVersionFailed")));
      return false;
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  async function createVariant(request: CreateVariantRequest): Promise<SkillVariant | null> {
    busy.value = true;
    busyText.value = t("messages.creatingVariant");
    try {
      const response = await invoke("create_skill_variant", { request }) as CreateVariantResponse;
      if (currentSkillPackage.value?.id === request.skillId) {
        await loadSkillPackage(request.skillId);
      }
      toast.success(t("messages.variantCreated", { name: response.variant.name }));
      return response.variant;
    } catch (err) {
      toast.error(getErrorMessage(err, t("errors.createVariantFailed")));
      return null;
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  async function updateVariant(request: UpdateVariantRequest): Promise<SkillVariant | null> {
    busy.value = true;
    busyText.value = t("messages.updatingVariant");
    try {
      const variant = await invoke("update_skill_variant", { request }) as SkillVariant;
      if (currentSkillPackage.value?.id === request.skillId) {
        await loadSkillPackage(request.skillId);
      }
      toast.success(t("messages.variantUpdated", { name: variant.name }));
      return variant;
    } catch (err) {
      toast.error(getErrorMessage(err, t("errors.updateVariantFailed")));
      return null;
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  async function deleteVariant(request: DeleteVariantRequest): Promise<boolean> {
    busy.value = true;
    busyText.value = t("messages.deletingVariant");
    try {
      await invoke("delete_skill_variant", { request });
      if (currentSkillPackage.value?.id === request.skillId) {
        await loadSkillPackage(request.skillId);
      }
      toast.success(t("messages.variantDeleted"));
      return true;
    } catch (err) {
      toast.error(getErrorMessage(err, t("errors.deleteVariantFailed")));
      return false;
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  function openVersionManagerModal(skillId: string) {
    showVersionManagerModal.value = true;
    void loadSkillPackage(skillId);
  }

  function closeVersionManagerModal() {
    showVersionManagerModal.value = false;
    currentSkillPackage.value = null;
  }

  function openVersionDiffModal() {
    showVersionDiffModal.value = true;
  }

  function closeVersionDiffModal() {
    showVersionDiffModal.value = false;
    currentVersionDiff.value = null;
  }

  onMounted(() => {
    refreshIdeOptions();
    loadMarketConfigs();
    void searchMarketplace(true);
    void scanLocalSkills();
  });

  return {
    // State
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

    // Actions
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
