import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { IdeSkill } from "./types";
import { getErrorMessage } from "./utils";

export type ToastFunction = (message: string) => void;
export type ErrorToastFunction = (message: string) => void;
export type ScanLocalSkillsFunction = () => Promise<boolean>;
export type TranslateFunction = (key: string, values?: Record<string, string | number>) => string;

export function useIdeAdoption(
  onSuccess: ToastFunction,
  onError: ErrorToastFunction,
  scanLocalSkills: ScanLocalSkillsFunction,
  t: TranslateFunction
) {
  const busy = ref(false);
  const busyText = ref("");

  async function adoptIdeSkill(skill: IdeSkill): Promise<void> {
    busy.value = true;
    busyText.value = t("messages.adopting");
    try {
      const message = (await invoke("adopt_ide_skill", {
        request: {
          targetPath: skill.path,
          ideLabel: skill.ide
        }
      })) as string;
      onSuccess(message);
      await scanLocalSkills();
    } catch (err) {
      onError(getErrorMessage(err, t("errors.adoptFailed")));
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  async function adoptManyIdeSkills(skills: IdeSkill[]): Promise<void> {
    if (skills.length === 0) return;
    busy.value = true;
    busyText.value = t("messages.adopting");
    let successCount = 0;
    let failCount = 0;
    const failedSkills: string[] = [];

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
          failedSkills.push(skill.name);
        }
      }

      if (successCount > 0 && failCount === 0) {
        onSuccess(t("messages.adoptedCount", { count: successCount }));
      } else if (successCount > 0 && failCount > 0) {
        onError(t("messages.adoptedPartial", { success: successCount, failed: failCount }));
        if (failedSkills.length > 0) {
          console.warn("[adoptManyIdeSkills] Failed to adopt:", failedSkills);
        }
      } else {
        onError(t("errors.adoptFailed"));
      }

      const scanResult = await scanLocalSkills();
      if (!scanResult) {
        console.warn("[adoptManyIdeSkills] scanLocalSkills failed after adopt");
      }
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  return {
    busy,
    busyText,
    adoptIdeSkill,
    adoptManyIdeSkills
  };
}
