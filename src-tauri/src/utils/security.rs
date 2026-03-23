use std::path::{Component, Path, PathBuf};

pub fn is_safe_relative_dir(rel: &str) -> bool {
    let trimmed = rel.trim();
    if trimmed.is_empty() {
        return false;
    }
    let path = Path::new(trimmed);
    if path.is_absolute() {
        return false;
    }
    for comp in path.components() {
        match comp {
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return false,
            _ => {}
        }
    }
    true
}

/// Checks if a path is a WSL UNC path
/// Examples: \\wsl$\Ubuntu\..., \\wsl.localhost\Ubuntu\...
pub fn is_wsl_path(path: &str) -> bool {
    let lower = path.trim().to_lowercase();
    lower.starts_with("\\\\wsl$\\") || lower.starts_with("\\\\wsl.localhost\\")
}

/// Validates if an absolute path is safe to use
/// - Unix absolute paths: /home/user/...
/// - Windows absolute paths: C:\Users\...
/// - WSL UNC paths: \\wsl$\Ubuntu\... or \\wsl.localhost\Ubuntu\...
pub fn is_safe_absolute_dir(path: &str) -> bool {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return false;
    }

    // WSL UNC paths
    if is_wsl_path(trimmed) {
        return true;
    }

    let p = Path::new(trimmed);
    if !p.is_absolute() {
        return false;
    }

    // On Unix, block dangerous system paths
    #[cfg(target_family = "unix")]
    {
        let dangerous = ["/etc", "/sys", "/proc", "/dev", "/root"];
        for d in dangerous {
            if trimmed == d || trimmed.starts_with(&format!("{}/", d)) {
                return false;
            }
        }
    }

    true
}

/// Validates a path - supports both relative and absolute paths
pub fn is_valid_ide_path(path: &str) -> bool {
    is_safe_relative_dir(path) || is_safe_absolute_dir(path)
}

/// Checks if the path is an absolute path (including WSL UNC)
pub fn is_absolute_ide_path(path: &str) -> bool {
    is_safe_absolute_dir(path)
}

pub fn is_within_directory(base: &Path, target: &Path) -> bool {
    let canonical_base = base.canonicalize().unwrap_or_else(|_| base.to_path_buf());

    let normalized_target = target.components().fold(PathBuf::new(), |mut acc, part| {
        if part == Component::ParentDir {
            acc.pop();
        } else if part != Component::CurDir {
            acc.push(part);
        }
        acc
    });

    let resolved_target = if target.is_absolute() {
        normalized_target
    } else {
        canonical_base.join(normalized_target)
    };

    resolved_target.starts_with(&canonical_base)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_safe_relative_dir_valid() {
        assert!(is_safe_relative_dir("skills"));
        assert!(is_safe_relative_dir("my-skills/test"));
        assert!(is_safe_relative_dir(".opencode/skills"));
    }

    #[test]
    fn test_is_safe_relative_dir_empty() {
        assert!(!is_safe_relative_dir(""));
        assert!(!is_safe_relative_dir("   "));
    }

    #[test]
    fn test_is_safe_relative_dir_absolute() {
        assert!(!is_safe_relative_dir("/home/user"));
        #[cfg(target_os = "windows")]
        {
            assert!(!is_safe_relative_dir("C:\\Users\\test"));
        }
    }

    #[test]
    fn test_is_safe_relative_dir_parent_dir() {
        assert!(!is_safe_relative_dir("../skills"));
        assert!(!is_safe_relative_dir("skills/../../../etc"));
    }

    #[test]
    fn test_is_safe_relative_dir_root() {
        assert!(!is_safe_relative_dir("/"));
    }

    #[test]
    fn test_is_wsl_path_valid() {
        assert!(is_wsl_path("\\\\wsl$\\Ubuntu\\home\\user"));
        assert!(is_wsl_path("\\\\wsl.localhost\\Ubuntu\\home"));
    }

    #[test]
    fn test_is_wsl_path_invalid() {
        assert!(!is_wsl_path("/home/user"));
        assert!(!is_wsl_path("C:\\Users\\test"));
        assert!(!is_wsl_path("\\\\server\\share"));
    }

    #[test]
    fn test_is_safe_absolute_dir_unix() {
        assert!(is_safe_absolute_dir("/home/user"));
        assert!(is_safe_absolute_dir("/tmp"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_is_safe_absolute_dir_windows() {
        assert!(is_safe_absolute_dir("C:\\Users\\test"));
        assert!(is_safe_absolute_dir("D:\\Projects"));
    }

    #[test]
    fn test_is_safe_absolute_dir_wsl() {
        assert!(is_safe_absolute_dir("\\\\wsl$\\Ubuntu\\home"));
        assert!(is_safe_absolute_dir("\\\\wsl.localhost\\Debian\\root"));
    }

    #[test]
    fn test_is_safe_absolute_dir_empty() {
        assert!(!is_safe_absolute_dir(""));
    }

    #[test]
    fn test_is_safe_absolute_dir_relative() {
        assert!(!is_safe_absolute_dir("skills"));
        assert!(!is_safe_absolute_dir("./skills"));
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn test_is_safe_absolute_dir_unix_dangerous() {
        assert!(!is_safe_absolute_dir("/etc"));
        assert!(!is_safe_absolute_dir("/etc/passwd"));
        assert!(!is_safe_absolute_dir("/sys"));
        assert!(!is_safe_absolute_dir("/proc"));
        assert!(!is_safe_absolute_dir("/dev"));
        assert!(!is_safe_absolute_dir("/root"));
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn test_is_safe_absolute_dir_unix_safe_under_dangerous() {
        assert!(is_safe_absolute_dir("/etc-backup"));
        assert!(is_safe_absolute_dir("/sysadmin"));
    }

    #[test]
    fn test_is_valid_ide_path() {
        assert!(is_valid_ide_path(".opencode/skills"));
        assert!(is_valid_ide_path("skills"));
        assert!(is_valid_ide_path("/home/user/skills"));
    }

    #[test]
    fn test_is_valid_ide_path_invalid() {
        assert!(!is_valid_ide_path("../skills"));
        assert!(!is_valid_ide_path(""));
    }

    #[test]
    fn test_is_absolute_ide_path() {
        assert!(is_absolute_ide_path("/home/user"));
        assert!(is_absolute_ide_path("\\\\wsl$\\Ubuntu"));
    }

    #[test]
    fn test_is_absolute_ide_path_relative() {
        assert!(!is_absolute_ide_path("skills"));
        assert!(!is_absolute_ide_path(".opencode/skills"));
    }

    #[test]
    fn test_is_within_directory_same_dir() {
        let base = Path::new("/home/user");
        let target = Path::new("/home/user");
        assert!(is_within_directory(base, target));
    }

    #[test]
    fn test_is_within_directory_subdir() {
        let base = Path::new("/home/user");
        let target = Path::new("/home/user/projects");
        assert!(is_within_directory(base, target));
    }

    #[test]
    fn test_is_within_directory_nested() {
        let base = Path::new("/home/user");
        let target = Path::new("/home/user/skills/manager/test");
        assert!(is_within_directory(base, target));
    }

    #[test]
    fn test_is_within_directory_outside() {
        let base = Path::new("/home/user");
        let target = Path::new("/home/other");
        assert!(!is_within_directory(base, target));
    }

    #[test]
    fn test_is_within_directory_sibling() {
        let base = Path::new("/home/user1");
        let target = Path::new("/home/user2");
        assert!(!is_within_directory(base, target));
    }

    #[test]
    fn test_is_within_directory_with_parent_dir() {
        let base = Path::new("/home/user");
        let target = Path::new("/home/user/../other");
        assert!(!is_within_directory(base, target));
    }

    #[test]
    fn test_is_within_directory_relative_path() {
        let base = Path::new("/home/user");
        let target = Path::new("projects");
        assert!(is_within_directory(base, target));
    }
}
