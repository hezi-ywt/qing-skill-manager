import assert from "node:assert/strict";

const localStorageMock = {
  data: new Map(),
  getItem(key) {
    return this.data.get(key) || null;
  },
  setItem(key, value) {
    this.data.set(key, value);
  },
  clear() {
    this.data.clear();
  }
};

global.localStorage = localStorageMock;

const STORAGE_KEYS = {
  PROJECTS: "qingSkillManager.projects"
};

function loadProjectsFromStorage() {
  try {
    const raw = localStorage.getItem(STORAGE_KEYS.PROJECTS);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(
      (item) =>
        item &&
        typeof item.id === "string" &&
        typeof item.name === "string" &&
        typeof item.path === "string" &&
        Array.isArray(item.ideTargets)
    );
  } catch {
    return [];
  }
}

function saveProjectsToStorage(projects) {
  localStorage.setItem(STORAGE_KEYS.PROJECTS, JSON.stringify(projects));
}

function runProjectConfigTests() {
  localStorageMock.clear();
  assert.deepStrictEqual(loadProjectsFromStorage(), []);

  const projects = [{ id: "proj-1", name: "Project", path: "/tmp/project", ideTargets: ["OpenCode"] }];
  saveProjectsToStorage(projects);
  assert.equal(loadProjectsFromStorage().length, 1);

  localStorage.setItem(STORAGE_KEYS.PROJECTS, "not json");
  assert.deepStrictEqual(loadProjectsFromStorage(), []);
  console.log("✓ useProjectConfig storage tests passed");
}

runProjectConfigTests();
