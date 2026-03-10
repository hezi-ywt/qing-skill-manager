/**
 * Utility functions for skills-manager
 */

/**
 * Validates if a path is a safe relative path (not absolute, no parent directory traversal)
 */
export function isSafeRelativePath(input: string): boolean {
  const trimmed = input.trim();
  if (!trimmed) return false;
  if (trimmed.startsWith("/") || /^[A-Za-z]:/i.test(trimmed) || trimmed.startsWith("\\")) {
    return false;
  }
  const parts = trimmed.split(/[\\/]+/);
  return parts.every((part) => part !== ".." && part !== "");
}

/**
 * Extracts error message from unknown error type
 */
export function getErrorMessage(err: unknown, fallback: string): string {
  if (err instanceof Error && err.message) return err.message;
  if (typeof err === "string" && err.trim()) return err;
  if (err && typeof err === "object") {
    const maybeMessage = (err as { message?: unknown }).message;
    if (typeof maybeMessage === "string" && maybeMessage.trim()) return maybeMessage;
  }
  return fallback;
}