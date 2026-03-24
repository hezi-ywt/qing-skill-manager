import assert from "node:assert/strict";
import {
  isSafeRelativePath,
  isSafeAbsolutePath,
  isValidIdePath,
  getErrorMessage,
  isNonNullObject,
  isValidArray,
  isNonEmptyString,
  validateArrayResponse,
  validateOverviewResponse,
  hasStringProperty,
  safeJsonParse
} from "./utils.ts";

function runUtilsTests() {
  assert.equal(isSafeRelativePath(".config/opencode/skills"), true);
  assert.equal(isSafeRelativePath("../etc/passwd"), false);
  assert.equal(isSafeAbsolutePath("/home/user"), true);
  assert.equal(isSafeAbsolutePath("/etc/passwd"), false);
  assert.equal(isValidIdePath(".config/opencode/skills"), true);
  assert.equal(getErrorMessage(new Error("boom"), "fallback"), "boom");
  assert.equal(isNonNullObject({}), true);
  assert.equal(isValidArray([]), true);
  assert.equal(isNonEmptyString("hello"), true);
  assert.equal(validateArrayResponse({ items: [1] }, "items").success, true);
  assert.equal(validateOverviewResponse({ managerSkills: [], ideSkills: [] }).success, true);
  assert.equal(hasStringProperty({ name: "ok" }, "name"), true);
  assert.equal(safeJsonParse('{"key":"value"}').success, true);
  console.log("✓ utils tests passed");
}

runUtilsTests();
