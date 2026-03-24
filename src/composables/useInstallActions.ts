import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { homeDir, join } from "@tauri-apps/api/path";
import type { LocalSkill, InstallResult, LinkTarget, IdeOption, ProjectConfig } from "./types";
import { isSafeRelativePath, isSafeAbsolutePath, getErrorMessage } from "./utils";
import { buildProjectCloneTargetPath } from "./constants";
import { loadLastInstallTargets, saveLastInstallTargets } from "./useIdeConfig";

export type ToastFunction = (message: string) => void;
export type ErrorToastFunction = (message: string) => void;
export type ScanLocalSkillsFunction = () => Promise<boolean>;
export type TranslateFunction = (key: string, values?: Record<string, string | number>) => string;

export function useInstallActions(
  ideOptions: { value: IdeOption[] },
  onSuccess: ToastFunction,
  onError: ErrorToastFunction,
  scanLocalSkills: ScanLocalSkillsFunction,
  t: TranslateFunction
) {
  const showInstallModal = ref(false);
  const installTargetSkills = ref<LocalSkill[]>([]);
  const installTargetIde = ref<string[]>([]);
  const installingId = ref<string | null>(null);
  const busy = ref(false);
  const busyText = ref("");

  async function buildInstallTargets(targetLabel: string): Promise<LinkTarget[]> {
    const target = ideOptions.value.find((option) => option.label === targetLabel);
    if (!target) return [];

    const dir = target.globalDir;

    if (isSafeAbsolutePath(dir)) {
      return [{ name: target.label, path: dir }];
    }

    if (!isSafeRelativePath(dir)) return [];

    const home = await homeDir();
    return [{ name: target.label, path: await join(home, dir) }];
  }

  async function buildProjectInstallTargets(projectPath: string, ideLabel: string): Promise<LinkTarget[]> {
    const target = ideOptions.value.find((option) => option.label === ideLabel);
    if (!target) return [];

    const skillPath = buildProjectCloneTargetPath(projectPath, ideLabel);
    if (!skillPath) return [];

    return [{ name: `${target.label} (${projectPath.split("/").pop()})`, path: skillPath }];
  }

  async function cloneSkillToIdeInternal(
    skill: LocalSkill,
    ideLabel: string,
    skipScan = false,
    suppressToast = false
  ): Promise<InstallResult> {
    const installTargets = await buildInstallTargets(ideLabel);
    if (installTargets.length === 0) {
      throw new Error(t("errors.selectValidIde"));
    }

    const result = (await invoke("clone_local_skill", {
      request: {
        skillPath: skill.path,
        skillName: skill.name,
        installTargets
      }
    })) as InstallResult;

    if (!suppressToast) {
      onSuccess(t("messages.handled", { installed: result.installed.length, skipped: result.skipped.length }));
    }
    if (!skipScan) {
      await scanLocalSkills();
    }
    return result;
  }

  async function cloneSkillToProjectInternal(
    skill: LocalSkill,
    projectPath: string,
    ideLabel: string,
    skipScan = false,
    suppressToast = false
  ): Promise<InstallResult> {
    const installTargets = await buildProjectInstallTargets(projectPath, ideLabel);
    if (installTargets.length === 0) {
      throw new Error(t("errors.selectValidIde"));
    }

    const result = (await invoke("clone_local_skill", {
      request: {
        skillPath: skill.path,
        skillName: skill.name,
        installTargets
      }
    })) as InstallResult;

    if (!suppressToast) {
      onSuccess(t("messages.handled", { installed: result.installed.length, skipped: result.skipped.length }));
    }
    if (!skipScan) {
      await scanLocalSkills();
    }
    return result;
  }

  function openInstallModal(skill: LocalSkill | LocalSkill[]): void {
    installTargetSkills.value = Array.isArray(skill) ? skill : [skill];
    const lastTargets = loadLastInstallTargets();
    const available = new Set(ideOptions.value.map((item) => item.label));
    installTargetIde.value = lastTargets.filter((label) => available.has(label));
    showInstallModal.value = true;
  }

  function updateInstallTargetIde(next: string[]): void {
    installTargetIde.value = next;
    saveLastInstallTargets(next);
  }

  function closeInstallModal(): void {
    showInstallModal.value = false;
    installTargetSkills.value = [];
  }

  async function confirmInstallToIde(
    installTarget: "ide" | "project",
    targetIds: string[],
    projects?: ProjectConfig[]
  ): Promise<void> {
    if (installTarget === "project") {
      if (!projects || projects.length === 0) {
        onError("No projects available");
        showInstallModal.value = false;
        installTargetSkills.value = [];
        return;
      }

      if (installTargetSkills.value.length === 0 || targetIds.length === 0) {
        onError(t("errors.selectAtLeastOne"));
        return;
      }

      if (installingId.value) return;
      installingId.value = installTargetSkills.value.length === 1 ? installTargetSkills.value[0].id : "__batch__";
      busy.value = true;
      busyText.value = t("messages.installing");

      try {
        let totalInstalled = 0;
        let totalSkipped = 0;
        const selectedProjects = projects.filter((project) => targetIds.includes(project.id));

        for (const skill of installTargetSkills.value) {
          for (const project of selectedProjects) {
            for (const ideLabel of project.ideTargets) {
              const result = await cloneSkillToProjectInternal(skill, project.path, ideLabel, true, true);
              totalInstalled += result.installed.length;
              totalSkipped += result.skipped.length;
            }
          }
        }

        onSuccess(t("messages.handled", { installed: totalInstalled, skipped: totalSkipped }));
        await scanLocalSkills();
        showInstallModal.value = false;
        installTargetSkills.value = [];
      } catch (err) {
        onError(getErrorMessage(err, t("errors.installFailed")));
      } finally {
        installingId.value = null;
        busy.value = false;
        busyText.value = "";
      }
      return;
    }

    if (installTargetSkills.value.length === 0 || targetIds.length === 0) {
      onError(t("errors.selectAtLeastOne"));
      return;
    }

    if (installingId.value) return;
    installingId.value = installTargetSkills.value.length === 1 ? installTargetSkills.value[0].id : "__batch__";
    busy.value = true;
    busyText.value = t("messages.installing");

    try {
      let totalInstalled = 0;
      let totalSkipped = 0;
      for (const skill of installTargetSkills.value) {
        for (const label of targetIds) {
          const result = await cloneSkillToIdeInternal(skill, label, true, true);
          totalInstalled += result.installed.length;
          totalSkipped += result.skipped.length;
        }
      }

      onSuccess(t("messages.handled", { installed: totalInstalled, skipped: totalSkipped }));
      await scanLocalSkills();
      showInstallModal.value = false;
      installTargetSkills.value = [];
    } catch (err) {
      onError(getErrorMessage(err, t("errors.installFailed")));
    } finally {
      installingId.value = null;
      busy.value = false;
      busyText.value = "";
    }
  }

  return {
    showInstallModal,
    installTargetSkills,
    installTargetIde,
    installingId,
    busy,
    busyText,
    openInstallModal,
    updateInstallTargetIde,
    closeInstallModal,
    confirmInstallToIde,
    cloneSkillToIdeInternal,
    cloneSkillToProjectInternal,
    buildInstallTargets,
    buildProjectInstallTargets
  };
}
