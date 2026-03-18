/**
 * Utility functions for skills-manager
 */

/**
 * Windows reserved names that cannot be used as file/directory names
 */
const WINDOWS_RESERVED_NAMES = [
  'CON', 'PRN', 'AUX', 'NUL',
  'COM1', 'COM2', 'COM3', 'COM4', 'COM5', 'COM6', 'COM7', 'COM8', 'COM9',
  'LPT1', 'LPT2', 'LPT3', 'LPT4', 'LPT5', 'LPT6', 'LPT7', 'LPT8', 'LPT9'
];

/**
 * Check if a name is a Windows reserved name
 */
function isWindowsReservedName(name: string): boolean {
  const upper = name.toUpperCase();
  // Check exact match
  if (WINDOWS_RESERVED_NAMES.includes(upper)) {
    return true;
  }
  // Check with extension (e.g., CON.txt, NUL.md)
  const base = upper.split('.')[0];
  if (WINDOWS_RESERVED_NAMES.includes(base)) {
    return true;
  }
  return false;
}

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
  if (parts.some((part) => part === ".." || part === "")) {
    return false;
  }
  // Check for Windows reserved names in any path component
  if (parts.some((part) => isWindowsReservedName(part))) {
    return false;
  }
  // Check for control characters
  if (/[\x00-\x1f\x7f]/.test(trimmed)) {
    return false;
  }
  return true;
}

/**
 * Checks if a path is a WSL UNC path
 * Examples: \\wsl$\Ubuntu\..., \\wsl.localhost\Ubuntu\...
 */
export function isWslPath(input: string): boolean {
  const trimmed = input.trim().toLowerCase();
  return trimmed.startsWith("\\\\wsl$\\") || trimmed.startsWith("\\\\wsl.localhost\\");
}

/**
 * Validates if an absolute path is safe to use
 * - Unix absolute paths: /home/user/...
 * - Windows absolute paths: C:\Users\...
 * - WSL UNC paths: \\wsl$\Ubuntu\... or \\wsl.localhost\Ubuntu\...
 */
export function isSafeAbsolutePath(input: string): boolean {
  const trimmed = input.trim();
  if (!trimmed) return false;

  // WSL UNC paths
  if (isWslPath(trimmed)) {
    return true;
  }

  // Unix absolute path
  if (trimmed.startsWith("/")) {
    // Disallow dangerous paths
    const dangerous = ["/etc", "/sys", "/proc", "/dev", "/root"];
    return !dangerous.some((d) => trimmed === d || trimmed.startsWith(d + "/"));
  }

  // Windows absolute path (e.g., C:\...)
  if (/^[A-Za-z]:[/\\]/.test(trimmed)) {
    return true;
  }

  return false;
}

/**
 * Validates a path - supports both relative and absolute paths
 */
export function isValidIdePath(input: string): boolean {
  return isSafeRelativePath(input) || isSafeAbsolutePath(input);
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