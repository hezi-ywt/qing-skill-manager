use crate::types::{
    AdoptIdeSkillRequest, AnalyzeConflictRequest, ConflictAnalysis, ConflictResolution,
    ConflictSeverity, ConflictType, CreateVariantRequest, CreateVariantResponse,
    CreateVersionRequest, CreateVersionResponse,
    DeleteLocalSkillRequest, DeleteStrategy, DeleteVariantRequest, DeleteVersionRequest,
    DeleteVersionResponse, GetSkillPackageRequest, GetSkillPackageResponse, IdeSkill, ImportRequest,
    InstallResult, LinkRequest, ListSkillPackagesResponse, LocalScanRequest, LocalSkill,
    MetadataChange, Overview, ProjectIdeDir, ProjectScanRequest, ProjectScanResult, ProjectSkill,
    ProjectSkillImportStatus, ProjectSkillScanResult, RenameVersionRequest,
    RenameVersionResponse, ResolutionAction, ResolutionSuggestion, ResolveConflictRequest,
    ResolveConflictResult, ScanProjectSkillsRequest, SetDefaultVersionRequest, SkillDiff,
    SkillPackage, SkillPackageSummary, SkillVariant, SkillVersion, SkillVersionMetadata,
    SkillVersionSource, UninstallRequest, UpdateVariantRequest,
};
use crate::utils::download::copy_dir_recursive;
use crate::utils::path::{normalize_path, resolve_canonical, sanitize_dir_name};
use crate::utils::security::{is_absolute_ide_path, is_valid_ide_path};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const VERSION_METADATA_FILE: &str = ".skills-manager-version.json";

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
struct StoredVersionMetadata {
    version: Option<String>,
    display_name: Option<String>,
    source_url: Option<String>,
    parent_version: Option<String>,
    deleted: Option<bool>,
}

#[derive(Debug, Clone)]
struct ParsedSkillMetadata {
    name: String,
    description: String,
    version: Option<String>,
    author: Option<String>,
    namespace: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
struct StoredPackageState {
    default_version: Option<String>,
    variants: Vec<SkillVariant>,
}

fn now_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn manager_versions_root(home: &Path) -> PathBuf {
    home.join(".skills-manager/versions")
}

fn build_skill_id(name: &str, namespace: Option<&str>) -> String {
    let safe_name = sanitize_dir_name(name);
    let safe_namespace = namespace
        .map(sanitize_dir_name)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "default".to_string());
    format!("{}_{}", safe_name, safe_namespace)
}

fn version_metadata_path(home: &Path, skill_id: &str, version_id: &str) -> PathBuf {
    manager_versions_root(home)
        .join(skill_id)
        .join(version_id)
        .join("metadata.json")
}

fn write_version_sidecar(skill_dir: &Path, sidecar: &StoredVersionMetadata) -> Result<(), String> {
    let path = skill_dir.join(VERSION_METADATA_FILE);
    let serialized = serde_json::to_string_pretty(sidecar).map_err(|err| err.to_string())?;
    fs::write(path, serialized).map_err(|err| err.to_string())
}

fn package_state_path(home: &Path, skill_id: &str) -> PathBuf {
    manager_versions_root(home).join(skill_id).join("package.json")
}

fn read_skill_metadata(skill_dir: &Path) -> (String, String) {
    let name = skill_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("skill")
        .to_string();

    let skill_file = skill_dir.join("SKILL.md");
    if !skill_file.exists() {
        return (name, String::new());
    }

    let content = fs::read_to_string(&skill_file).unwrap_or_default();
    let lines = content.lines();

    let mut frontmatter_name: Option<String> = None;
    let mut description = String::new();

    let mut in_frontmatter = false;
    let mut frontmatter_closed = false;
    for line in lines {
        let trimmed = line.trim();
        if trimmed == "---" {
            if !in_frontmatter {
                in_frontmatter = true;
                continue;
            }
            in_frontmatter = false;
            frontmatter_closed = true;
            continue;
        }
        if in_frontmatter {
            if let Some(value) = trimmed.strip_prefix("name:") {
                frontmatter_name = Some(value.trim().to_string());
            }
            continue;
        }
        if (frontmatter_closed || frontmatter_name.is_none())
            && description.is_empty()
            && !trimmed.is_empty()
            && !trimmed.starts_with('#')
        {
            description = trimmed.to_string();
        }
    }

    let final_name = frontmatter_name.unwrap_or(name);
    (final_name, description)
}

fn parse_skill_metadata(skill_dir: &Path) -> ParsedSkillMetadata {
    let (name, description) = read_skill_metadata(skill_dir);
    let content = fs::read_to_string(skill_dir.join("SKILL.md")).unwrap_or_default();
    let mut version = None;
    let mut author = None;
    let mut namespace = None;
    let mut in_frontmatter = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "---" {
            in_frontmatter = !in_frontmatter;
            continue;
        }
        if !in_frontmatter {
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("version:") {
            version = Some(value.trim().to_string());
        } else if let Some(value) = trimmed.strip_prefix("author:") {
            author = Some(value.trim().to_string());
        } else if let Some(value) = trimmed.strip_prefix("namespace:") {
            namespace = Some(value.trim().to_string());
        }
    }

    ParsedSkillMetadata {
        name,
        description,
        version,
        author,
        namespace,
    }
}

fn simple_hash(input: &str) -> String {
    let mut hash: u64 = 1469598103934665603;
    for byte in input.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("{:016x}", hash)
}

fn skill_content_hash(skill_dir: &Path) -> String {
    let content = fs::read_to_string(skill_dir.join("SKILL.md")).unwrap_or_default();
    simple_hash(&content)
}

fn read_version_sidecar(skill_dir: &Path) -> StoredVersionMetadata {
    let metadata_path = skill_dir.join(VERSION_METADATA_FILE);
    if !metadata_path.exists() {
        return StoredVersionMetadata::default();
    }

    fs::read_to_string(metadata_path)
        .ok()
        .and_then(|content| serde_json::from_str::<StoredVersionMetadata>(&content).ok())
        .unwrap_or_default()
}

fn build_skill_version(skill_dir: &Path, source: SkillVersionSource) -> SkillVersion {
    let metadata = parse_skill_metadata(skill_dir);
    let sidecar = read_version_sidecar(skill_dir);
    let content_hash = skill_content_hash(skill_dir);
    let version_label = sidecar
        .version
        .clone()
        .or(metadata.version.clone())
        .unwrap_or_else(|| "1.0.0".to_string());
    let skill_id = build_skill_id(&metadata.name, metadata.namespace.as_deref());
    let version_id = format!("{}_{}", sanitize_dir_name(&version_label), &content_hash[..8]);

    SkillVersion {
        id: version_id,
        skill_id,
        version: version_label.clone(),
        display_name: sidecar.display_name.unwrap_or(version_label),
        content_hash,
        created_at: now_timestamp(),
        source,
        source_url: sidecar.source_url,
        parent_version: sidecar.parent_version,
        is_active: !sidecar.deleted.unwrap_or(false),
        metadata: SkillVersionMetadata {
            name: metadata.name,
            description: metadata.description,
            author: metadata.author,
            namespace: metadata.namespace,
        },
    }
}

fn write_version_metadata(home: &Path, version: &SkillVersion) -> Result<(), String> {
    let path = version_metadata_path(home, &version.skill_id, &version.id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let serialized = serde_json::to_string_pretty(version).map_err(|err| err.to_string())?;
    fs::write(path, serialized).map_err(|err| err.to_string())
}

fn read_package_state(home: &Path, skill_id: &str) -> StoredPackageState {
    let path = package_state_path(home, skill_id);
    if !path.exists() {
        return StoredPackageState::default();
    }

    fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<StoredPackageState>(&content).ok())
        .unwrap_or_default()
}

fn write_package_state(home: &Path, skill_id: &str, state: &StoredPackageState) -> Result<(), String> {
    let path = package_state_path(home, skill_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let serialized = serde_json::to_string_pretty(state).map_err(|err| err.to_string())?;
    fs::write(path, serialized).map_err(|err| err.to_string())
}

fn collect_versions_for_skill(base: &Path, skill_id: &str) -> Vec<(PathBuf, SkillVersion)> {
    let Some(home) = dirs::home_dir() else {
        return Vec::new();
    };

    let mut versions = Vec::new();
    let entries = match fs::read_dir(base) {
        Ok(entries) => entries,
        Err(_) => return versions,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() || !path.join("SKILL.md").exists() {
            continue;
        }

        let version = version_summary_for_skill(&home, &path);
        if version.skill_id == skill_id {
            versions.push((path, version));
        }
    }

    versions.sort_by(|left, right| right.1.created_at.cmp(&left.1.created_at));
    versions
}

fn version_summary_for_skill(home: &Path, skill_dir: &Path) -> SkillVersion {
    let version = build_skill_version(skill_dir, SkillVersionSource::Migration);
    let _ = write_version_metadata(home, &version);
    version
}

fn build_skill_diff(base: &SkillVersion, incoming: &SkillVersion) -> SkillDiff {
    let mut metadata_changes = Vec::new();

    if base.metadata.description != incoming.metadata.description {
        metadata_changes.push(MetadataChange {
            field: "description".to_string(),
            old_value: Some(base.metadata.description.clone()),
            new_value: Some(incoming.metadata.description.clone()),
        });
    }
    if base.metadata.author != incoming.metadata.author {
        metadata_changes.push(MetadataChange {
            field: "author".to_string(),
            old_value: base.metadata.author.clone(),
            new_value: incoming.metadata.author.clone(),
        });
    }
    if base.version != incoming.version {
        metadata_changes.push(MetadataChange {
            field: "version".to_string(),
            old_value: Some(base.version.clone()),
            new_value: Some(incoming.version.clone()),
        });
    }

    let similarity_score = if base.content_hash == incoming.content_hash {
        1.0
    } else if metadata_changes.len() <= 1 {
        0.82
    } else if metadata_changes.len() <= 3 {
        0.55
    } else {
        0.25
    };

    SkillDiff {
        from_version: base.id.clone(),
        to_version: incoming.id.clone(),
        files_changed: vec!["SKILL.md".to_string()],
        additions: incoming.metadata.description.lines().count(),
        deletions: base.metadata.description.lines().count(),
        content_diff: Some(format!(
            "--- existing\n+++ incoming\n- version: {}\n+ version: {}\n- description: {}\n+ description: {}",
            base.version,
            incoming.version,
            base.metadata.description,
            incoming.metadata.description
        )),
        metadata_changes,
        similarity_score,
    }
}

fn classify_conflict(
    diff: &SkillDiff,
) -> (ConflictType, ConflictSeverity, bool, Vec<ResolutionSuggestion>) {
    if (diff.similarity_score - 1.0).abs() < f64::EPSILON {
        return (
            ConflictType::Identical,
            ConflictSeverity::None,
            true,
            vec![ResolutionSuggestion {
                action: ResolutionAction::FastForward,
                description: "Identical content; keep the existing version".to_string(),
                confidence: 1.0,
            }],
        );
    }

    if diff.metadata_changes.len() <= 1 {
        return (
            ConflictType::Patch,
            ConflictSeverity::Minor,
            true,
            vec![ResolutionSuggestion {
                action: ResolutionAction::CreateVersion,
                description: "Small metadata-only difference; add as a new version".to_string(),
                confidence: 0.9,
            }],
        );
    }

    if diff.similarity_score >= 0.5 {
        return (
            ConflictType::Minor,
            ConflictSeverity::Major,
            true,
            vec![
                ResolutionSuggestion {
                    action: ResolutionAction::CreateVersion,
                    description: "Moderate changes detected; store as a new version".to_string(),
                    confidence: 0.78,
                },
                ResolutionSuggestion {
                    action: ResolutionAction::CreateVariant,
                    description: "Keep a separate named variant if both should remain discoverable".to_string(),
                    confidence: 0.64,
                },
            ],
        );
    }

    (
        ConflictType::Fork,
        ConflictSeverity::Breaking,
        false,
        vec![ResolutionSuggestion {
            action: ResolutionAction::InteractiveMerge,
            description: "Substantial divergence detected; compare and resolve manually".to_string(),
            confidence: 0.71,
        }],
    )
}

fn package_from_skill_dir(home: &Path, manager_dir: &Path, skill_dir: &Path) -> SkillPackage {
    let primary_version = build_skill_version(skill_dir, SkillVersionSource::Migration);
    let mut versions: Vec<SkillVersion> = collect_versions_for_skill(manager_dir, &primary_version.skill_id)
        .into_iter()
        .map(|(_, version)| version)
        .collect();

    if versions.is_empty() {
        versions.push(primary_version.clone());
    }

    let mut state = read_package_state(home, &primary_version.skill_id);
    if state.variants.is_empty() {
        state.variants.push(SkillVariant {
            id: format!("{}-default", primary_version.skill_id),
            name: "default".to_string(),
            current_version: state
                .default_version
                .clone()
                .unwrap_or_else(|| primary_version.id.clone()),
            created_at: now_timestamp(),
            description: Some("Default tracked version".to_string()),
        });
    }

    let default_version = state
        .default_version
        .clone()
        .or_else(|| versions.first().map(|version| version.id.clone()))
        .unwrap_or_else(|| primary_version.id.clone());

    SkillPackage {
        id: primary_version.skill_id.clone(),
        name: primary_version.metadata.name.clone(),
        namespace: primary_version
            .metadata
            .namespace
            .clone()
            .unwrap_or_else(|| "default".to_string()),
        default_version,
        versions,
        variants: state.variants,
        created_at: now_timestamp(),
        updated_at: now_timestamp(),
    }
}

fn collect_skills_from_dir(base: &Path, source: &str, ide: Option<&str>) -> Vec<LocalSkill> {
    let mut skills = Vec::new();
    if !base.exists() {
        return skills;
    }

    let entries = match fs::read_dir(base) {
        Ok(entries) => entries,
        Err(_) => return skills,
    };

    for entry in entries {
        let entry = match entry {
            Ok(item) => item,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_dir() || !path.join("SKILL.md").exists() {
            continue;
        }
        let (name, description) = read_skill_metadata(&path);
        let version = dirs::home_dir().map(|home| version_summary_for_skill(&home, &path));
        skills.push(LocalSkill {
            id: path.display().to_string(),
            name,
            description,
            path: path.display().to_string(),
            source: source.to_string(),
            ide: ide.map(|value| value.to_string()),
            used_by: Vec::new(),
            version_count: usize::from(version.is_some()),
            current_version: version,
        });
    }

    skills
}

fn collect_ide_skills(
    base: &Path,
    ide_label: &str,
    manager_map: &[(PathBuf, usize)],
    manager_skills: &mut [LocalSkill],
) -> Vec<IdeSkill> {
    let mut skills = Vec::new();
    if !base.exists() {
        return skills;
    }

    let entries = match fs::read_dir(base) {
        Ok(entries) => entries,
        Err(_) => return skills,
    };

    for entry in entries {
        let entry = match entry {
            Ok(item) => item,
            Err(_) => continue,
        };
        let path = entry.path();
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        let link_target = fs::read_link(&path).ok();
        if !metadata.is_dir() && link_target.is_none() {
            continue;
        }

        let skill_dir = path.as_path();
        let has_skill_file = skill_dir.join("SKILL.md").exists();
        if !has_skill_file && link_target.is_none() {
            continue;
        }

        let name = if has_skill_file {
            read_skill_metadata(skill_dir).0
        } else {
            skill_dir
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("skill")
                .to_string()
        };

        let path = skill_dir.to_path_buf();
        let mut managed = false;
        let source = if let Some(link_target) = link_target {
            let absolute_target = if link_target.is_relative() {
                if let Some(parent) = path.parent() {
                    parent.join(&link_target)
                } else {
                    link_target.clone()
                }
            } else {
                link_target
            };

            if let Some(target) = resolve_canonical(&absolute_target) {
                for (manager_path, idx) in manager_map {
                    if *manager_path == target {
                        managed = true;
                        if let Some(skill) = manager_skills.get_mut(*idx) {
                            if !skill.used_by.contains(&ide_label.to_string()) {
                                skill.used_by.push(ide_label.to_string());
                            }
                        }
                        break;
                    }
                }
            }
            "link"
        } else {
            "local"
        };

        skills.push(IdeSkill {
            id: path.display().to_string(),
            name,
            path: path.display().to_string(),
            ide: ide_label.to_string(),
            source: source.to_string(),
            managed,
        });
    }

    skills
}

fn remove_path(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|err| err.to_string())?;
    if metadata.file_type().is_symlink() {
        // `path.is_dir()` follows symlinks and may report true for a symlink-to-dir.
        // Removing such a symlink with `remove_dir` triggers ENOTDIR on macOS.
        fs::remove_file(path)
            .or_else(|_| fs::remove_dir(path))
            .map_err(|err| err.to_string())
    } else if metadata.is_dir() {
        fs::remove_dir_all(path).map_err(|err| err.to_string())
    } else {
        fs::remove_file(path).map_err(|err| err.to_string())
    }
}

fn is_symlink_to(path: &Path, target: &Path) -> bool {
    match (resolve_canonical(path), resolve_canonical(target)) {
        (Some(link_target), Some(expected_target)) => link_target == expected_target,
        _ => false,
    }
}

fn create_symlink_dir(target: &Path, link: &Path) -> Result<(), String> {
    #[cfg(target_family = "unix")]
    {
        std::os::unix::fs::symlink(target, link).map_err(|err| err.to_string())
    }
    #[cfg(target_family = "windows")]
    {
        std::os::windows::fs::symlink_dir(target, link).map_err(|err| err.to_string())
    }
}

#[cfg(target_family = "windows")]
fn create_junction_dir(target: &Path, link: &Path) -> Result<(), String> {
    use std::process::Command;

    fn to_cmd_path(path: &Path) -> String {
        path.to_string_lossy().replace('/', "\\")
    }

    fn validate_path(path: &str) -> Result<(), String> {
        let dangerous_chars = ['|', '^', '<', '>', '%', '!', '"', '&', '(', ')', ';'];
        for ch in dangerous_chars {
            if path.contains(ch) {
                return Err(format!("Path contains dangerous character: '{}'", ch));
            }
        }
        Ok(())
    }

    let target = to_cmd_path(target);
    let link = to_cmd_path(link);

    validate_path(&target)?;
    validate_path(&link)?;

    let output = Command::new("cmd")
        .args(["/C", "mklink", "/J", &link, &target])
        .output()
        .map_err(|err| err.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let detail = if !stderr.is_empty() {
            stderr
        } else if !stdout.is_empty() {
            stdout
        } else {
            "unknown error".to_string()
        };
        Err(format!("mklink /J failed: {}", detail))
    }
}

#[tauri::command]
pub fn link_local_skill(request: LinkRequest) -> Result<InstallResult, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let normalized_home = normalize_path(&home);
    let manager_root_raw = home.join(".skills-manager/skills");
    let manager_root =
        resolve_canonical(&manager_root_raw).unwrap_or_else(|| normalize_path(&manager_root_raw));

    let skill_path = PathBuf::from(&request.skill_path);
    let skill_canon = resolve_canonical(&skill_path)
        .ok_or_else(|| "Local skill path does not exist".to_string())?;
    if !skill_canon.starts_with(&manager_root) {
        return Err("Local skill path must stay inside Skills Manager storage".to_string());
    }
    let skill_path = skill_canon;

    let safe_name = sanitize_dir_name(&request.skill_name);

    let mut linked = Vec::new();
    let mut skipped = Vec::new();

    for target in request.link_targets {
        let target_base = PathBuf::from(&target.path);
        let normalized_target = normalize_path(&target_base);
        if !normalized_target.starts_with(&normalized_home) {
            return Err(format!(
                "Target directory is outside the home directory: {}",
                target.name
            ));
        }

        // Normalize resolved paths before comparison so Windows verbatim prefixes do not
        // trigger false-positive symlink attack errors.
        let target_canon =
            resolve_canonical(&target_base).unwrap_or_else(|| normalized_target.clone());
        if !target_canon.starts_with(&normalized_home) {
            return Err(format!(
                "Target directory failed the symlink safety check: {}",
                target.name
            ));
        }

        fs::create_dir_all(&target_base).map_err(|err| err.to_string())?;
        let link_path = target_base.join(&safe_name);

        if fs::symlink_metadata(&link_path).is_ok() {
            if is_symlink_to(&link_path, &skill_path) {
                skipped.push(format!("{}: already linked", target.name));
                continue;
            }
            skipped.push(format!("{}: target already exists", target.name));
            continue;
        }

        let mut linked_done = false;
        let mut link_errors = Vec::new();

        match create_symlink_dir(&skill_path, &link_path) {
            Ok(()) => {
                linked.push(format!("{}: {}", target.name, link_path.display()));
                linked_done = true;
            }
            Err(err) => link_errors.push(format!("symlink: {}", err)),
        }

        #[cfg(target_family = "windows")]
        if !linked_done {
            match create_junction_dir(&skill_path, &link_path) {
                Ok(()) => {
                    linked.push(format!("{}: junction {}", target.name, link_path.display()));
                    linked_done = true;
                }
                Err(err) => link_errors.push(format!("junction: {}", err)),
            }
        }

        if !linked_done {
            let detail = if link_errors.is_empty() {
                "unknown error".to_string()
            } else {
                link_errors.join("; ")
            };
            return Err(format!(
                "Failed to create a link for {} in {}: {}",
                request.skill_name, target.name, detail
            ));
        }
    }

    Ok(InstallResult {
        installed_path: skill_path.display().to_string(),
        linked,
        skipped,
    })
}

#[tauri::command]
pub fn scan_overview(request: LocalScanRequest) -> Result<Overview, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;

    let manager_dir = home.join(".skills-manager/skills");
    let mut manager_skills = collect_skills_from_dir(&manager_dir, "manager", None);

    // Resolve IDE directories: absolute paths are used directly, relative paths are joined with home
    let ide_dirs: Vec<(String, PathBuf)> = if request.ide_dirs.is_empty() {
        vec![
            (
                "Antigravity".to_string(),
                home.join(".gemini/antigravity/skills"),
            ),
            ("Claude".to_string(), home.join(".claude/skills")),
            ("CodeBuddy".to_string(), home.join(".codebuddy/skills")),
            ("Codex".to_string(), home.join(".codex/skills")),
            ("Cursor".to_string(), home.join(".cursor/skills")),
            ("Kiro".to_string(), home.join(".kiro/skills")),
            ("Qoder".to_string(), home.join(".qoder/skills")),
            ("Trae".to_string(), home.join(".trae/skills")),
            ("VSCode".to_string(), home.join(".github/skills")),
            ("Windsurf".to_string(), home.join(".windsurf/skills")),
        ]
    } else {
        request
            .ide_dirs
            .iter()
            .map(|item| {
                if !is_valid_ide_path(&item.relative_dir) {
                    return Err(format!("Invalid IDE directory: {}", item.label));
                }
                // Absolute path: use directly
                if is_absolute_ide_path(&item.relative_dir) {
                    Ok((item.label.clone(), PathBuf::from(&item.relative_dir)))
                } else {
                    // Relative path: join with home directory
                    Ok((item.label.clone(), home.join(&item.relative_dir)))
                }
            })
            .collect::<Result<Vec<_>, String>>()?
    };

    let mut ide_skills: Vec<IdeSkill> = Vec::new();

    let mut manager_map: Vec<(PathBuf, usize)> = Vec::new();
    for (idx, skill) in manager_skills.iter().enumerate() {
        if let Some(path) = resolve_canonical(Path::new(&skill.path)) {
            manager_map.push((path, idx));
        }
    }

    for (label, dir) in &ide_dirs {
        ide_skills.extend(collect_ide_skills(
            dir,
            label,
            &manager_map,
            &mut manager_skills,
        ));
    }

    if let Some(project) = request.project_dir {
        let base = PathBuf::from(project);
        for (label, dir) in &ide_dirs {
            // For absolute paths, also check the same path under project
            // For relative paths, join with project directory
            let project_dir = if dir.is_absolute() {
                dir.clone()
            } else {
                base.join(dir)
            };
            ide_skills.extend(collect_ide_skills(
                &project_dir,
                label,
                &manager_map,
                &mut manager_skills,
            ));
        }
    }

    Ok(Overview {
        manager_skills,
        ide_skills,
    })
}

#[tauri::command]
pub fn uninstall_skill(request: UninstallRequest) -> Result<String, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let mut allowed_roots = vec![home.join(".skills-manager/skills")];

    let ide_dirs: Vec<String> = if request.ide_dirs.is_empty() {
        vec![
            ".gemini/antigravity/skills".to_string(),
            ".claude/skills".to_string(),
            ".codebuddy/skills".to_string(),
            ".codex/skills".to_string(),
            ".cursor/skills".to_string(),
            ".kiro/skills".to_string(),
            ".qoder/skills".to_string(),
            ".trae/skills".to_string(),
            ".github/skills".to_string(),
            ".windsurf/skills".to_string(),
        ]
    } else {
        request
            .ide_dirs
            .iter()
            .map(|item| item.relative_dir.clone())
            .collect()
    };

    for dir in &ide_dirs {
        if !is_valid_ide_path(dir) {
            return Err("Invalid IDE directory".to_string());
        }
        // Absolute path: add directly to allowed roots
        if is_absolute_ide_path(dir) {
            allowed_roots.push(PathBuf::from(dir));
        } else {
            // Relative path: join with home directory
            allowed_roots.push(home.join(dir));
        }
    }
    if let Some(project) = request.project_dir {
        let base = PathBuf::from(project);
        allowed_roots.push(base.join(".codex/skills"));
        allowed_roots.push(base.join(".trae/skills"));
        allowed_roots.push(base.join(".opencode/skill"));
        allowed_roots.push(base.join(".skills-manager/skills"));
    }

    let target = PathBuf::from(&request.target_path);
    let parent = target.parent().unwrap_or(Path::new(&request.target_path));
    let parent_canon = resolve_canonical(parent).unwrap_or_else(|| normalize_path(parent));
    let allowed_roots_canon: Vec<PathBuf> = allowed_roots
        .iter()
        .map(|root| resolve_canonical(root).unwrap_or_else(|| normalize_path(root)))
        .collect();
    let allowed = allowed_roots_canon
        .iter()
        .any(|root| parent_canon.starts_with(root));
    if !allowed {
        return Err("Target path is outside the allowed directories".to_string());
    }

    let metadata = fs::symlink_metadata(&target).map_err(|err| err.to_string())?;
    if metadata.file_type().is_symlink() {
        // `target.is_dir()` follows symlinks and may report true for a symlink-to-dir.
        // Removing such a symlink with `remove_dir` triggers ENOTDIR/ENOTEMPTY on macOS.
        fs::remove_file(&target)
            .or_else(|_| fs::remove_dir(&target))
            .map_err(|err| err.to_string())?;
        return Ok("Link removed".to_string());
    }

    fs::remove_dir_all(&target).map_err(|err| err.to_string())?;
    Ok("Directory removed".to_string())
}

#[tauri::command]
pub fn import_local_skill(request: ImportRequest) -> Result<String, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let manager_dir = home.join(".skills-manager/skills");

    let source_path = PathBuf::from(&request.source_path);
    if !source_path.exists() {
        return Err("Source path does not exist".to_string());
    }

    if !source_path.join("SKILL.md").exists() {
        return Err("The selected directory does not contain SKILL.md".to_string());
    }

    let (name, _) = read_skill_metadata(&source_path);
    let safe_name = sanitize_dir_name(&name);
    let target_dir = manager_dir.join(&safe_name);

    if target_dir.exists() {
        return Err(format!("Target skill already exists: {}", safe_name));
    }

    fs::create_dir_all(&target_dir).map_err(|err| err.to_string())?;
    copy_dir_recursive(&source_path, &target_dir)?;

    Ok(format!("Imported skill: {}", name))
}

#[tauri::command]
pub fn adopt_ide_skill(request: AdoptIdeSkillRequest) -> Result<String, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory".to_string())?;
    let normalized_home = normalize_path(&home);
    let manager_root = home.join(".skills-manager/skills");
    fs::create_dir_all(&manager_root).map_err(|err| err.to_string())?;

    let target = PathBuf::from(&request.target_path);
    let normalized_target = normalize_path(&target);
    if !normalized_target.starts_with(&normalized_home) {
        return Err("IDE skill path must stay inside the home directory".to_string());
    }

    fs::symlink_metadata(&target).map_err(|_| "IDE skill path does not exist".to_string())?;
    let target_canon = resolve_canonical(&target);

    let (name, has_skill_file) = if let Some(path) = target_canon.as_ref() {
        (read_skill_metadata(path).0, path.join("SKILL.md").exists())
    } else {
        (
            target
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("skill")
                .to_string(),
            false,
        )
    };

    let safe_name = sanitize_dir_name(&name);
    let manager_target = manager_root.join(&safe_name);

    if manager_target.exists() {
        let manager_canon = resolve_canonical(&manager_target)
            .ok_or_else(|| "Managed skill path does not exist".to_string())?;
        if target_canon
            .as_ref()
            .is_some_and(|target_path| *target_path == manager_canon)
        {
            return Ok(format!("{} is already managed", name));
        }
    } else {
        let source_dir = target_canon
            .as_ref()
            .ok_or_else(|| "IDE skill path does not exist".to_string())?;
        if !has_skill_file {
            return Err("Target directory does not contain SKILL.md".to_string());
        }
        copy_dir_recursive(source_dir, &manager_target)?;
    }

    remove_path(&target)?;

    let mut linked_done = false;
    let mut link_errors = Vec::new();

    match create_symlink_dir(&manager_target, &target) {
        Ok(()) => linked_done = true,
        Err(err) => link_errors.push(format!("symlink: {}", err)),
    }

    #[cfg(target_family = "windows")]
    if !linked_done {
        match create_junction_dir(&manager_target, &target) {
            Ok(()) => linked_done = true,
            Err(err) => link_errors.push(format!("junction: {}", err)),
        }
    }

    if !linked_done {
        copy_dir_recursive(&manager_target, &target)?;
        let detail = if link_errors.is_empty() {
            "unknown error".to_string()
        } else {
            link_errors.join("; ")
        };
        return Err(format!(
            "Managed {} in Skills Manager, but failed to create a link for {}. Restored a local copy instead. {}",
            name, request.ide_label, detail
        ));
    }

    Ok(format!(
        "Managed {} and re-linked it to {}",
        name, request.ide_label
    ))
}

#[tauri::command]
pub fn delete_local_skills(request: DeleteLocalSkillRequest) -> Result<String, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let manager_root = resolve_canonical(&home.join(".skills-manager/skills"))
        .unwrap_or_else(|| normalize_path(&home.join(".skills-manager/skills")));

    if request.target_paths.is_empty() {
        return Err("No skills were provided for deletion".to_string());
    }

    let mut deleted = 0usize;

    for raw_path in request.target_paths {
        let target = PathBuf::from(&raw_path);
        let canonical =
            resolve_canonical(&target).ok_or_else(|| "Target skill does not exist".to_string())?;
        if !canonical.starts_with(&manager_root) {
            return Err("Only Skills Manager local skills can be deleted".to_string());
        }
        if canonical == manager_root {
            return Err("Refusing to delete the skills root directory".to_string());
        }
        if !canonical.join("SKILL.md").exists() {
            return Err("Refusing to delete a directory without SKILL.md".to_string());
        }

        fs::remove_dir_all(&canonical).map_err(|err| err.to_string())?;
        deleted += 1;
    }

    Ok(format!("Deleted {} skills", deleted))
}

#[tauri::command]
pub fn scan_project_ide_dirs(request: ProjectScanRequest) -> Result<ProjectScanResult, String> {
    let project_dir = PathBuf::from(&request.project_dir);

    if !project_dir.exists() {
        return Err("Project directory does not exist".to_string());
    }

    let ide_dir_patterns = [
        (".gemini/antigravity/skills", "Antigravity"),
        (".claude/skills", "Claude Code"),
        (".codebuddy/skills", "CodeBuddy"),
        (".codex/skills", "Codex"),
        (".cursor/skills", "Cursor"),
        (".kiro/skills", "Kiro"),
        (".openclaw/skills", "OpenClaw"),
        (".config/opencode/skills", "OpenCode"),
        (".qoder/skills", "Qoder"),
        (".trae/skills", "Trae"),
        (".github/skills", "VSCode"),
        (".windsurf/skills", "Windsurf"),
    ];

    let mut detected_ide_dirs = Vec::new();

    for (relative_path, label) in ide_dir_patterns.iter() {
        let ide_path = project_dir.join(relative_path);
        if ide_path.exists() && ide_path.is_dir() {
            detected_ide_dirs.push(ProjectIdeDir {
                label: label.to_string(),
                relative_dir: relative_path.to_string(),
                absolute_path: ide_path.display().to_string(),
            });
        }
    }

    Ok(ProjectScanResult {
        project_dir: request.project_dir,
        detected_ide_dirs,
    })
}

#[tauri::command]
pub fn scan_project_opencode_skills(
    request: ScanProjectSkillsRequest,
) -> Result<ProjectSkillScanResult, String> {
    let project_dir = PathBuf::from(&request.project_dir);
    let manager_root = PathBuf::from(&request.manager_root);

    let opencode_path = project_dir.join(".opencode/skills");
    if !opencode_path.exists() || !opencode_path.is_dir() {
        return Ok(ProjectSkillScanResult {
            project_path: request.project_dir,
            skills: Vec::new(),
            new_count: 0,
            duplicate_count: 0,
            conflict_count: 0,
        });
    }

    let existing_skills = collect_skills_from_dir(&manager_root, "manager", None);
    let existing_names: std::collections::HashMap<String, LocalSkill> = existing_skills
        .into_iter()
        .map(|s| (s.name.clone(), s))
        .collect();

    let mut skills = Vec::new();
    let mut new_count = 0;
    let mut duplicate_count = 0;
    let mut conflict_count = 0;

    let entries = match fs::read_dir(&opencode_path) {
        Ok(entries) => entries,
        Err(_) => {
            return Ok(ProjectSkillScanResult {
                project_path: request.project_dir,
                skills: Vec::new(),
                new_count: 0,
                duplicate_count: 0,
                conflict_count: 0,
            })
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(item) => item,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_dir() || !path.join("SKILL.md").exists() {
            continue;
        }

        let (name, description) = read_skill_metadata(&path);
        let incoming_version = build_skill_version(&path, SkillVersionSource::Project);
        let status = if let Some(existing) = existing_names.get(&name) {
            if existing
                .current_version
                .as_ref()
                .is_some_and(|version| version.content_hash == incoming_version.content_hash)
            {
                ProjectSkillImportStatus::Duplicate
            } else {
                ProjectSkillImportStatus::Conflict
            }
        } else {
            ProjectSkillImportStatus::New
        };

        match &status {
            ProjectSkillImportStatus::New => new_count += 1,
            ProjectSkillImportStatus::Duplicate => duplicate_count += 1,
            ProjectSkillImportStatus::Conflict => conflict_count += 1,
        }

        skills.push(ProjectSkill {
            id: path.display().to_string(),
            name: name.clone(),
            description,
            path: path.display().to_string(),
            status: status.clone(),
            existing_registry_skill: existing_names.get(&name).cloned(),
            current_version: Some(incoming_version),
        });
    }

    Ok(ProjectSkillScanResult {
        project_path: request.project_dir,
        skills,
        new_count,
        duplicate_count,
        conflict_count,
    })
}

#[tauri::command]
pub fn resolve_skill_conflict(
    request: ResolveConflictRequest,
) -> Result<ResolveConflictResult, String> {
    let skill_path = PathBuf::from(&request.project_skill_path);
    let skill_name = skill_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("skill")
        .to_string();

    let result = match request.resolution {
        ConflictResolution::Keep => ResolveConflictResult {
            success: true,
            skill_id: None,
            action: "kept_existing".to_string(),
        },
        ConflictResolution::Overwrite => {
            let safe_name = sanitize_dir_name(&skill_name);
            ResolveConflictResult {
                success: true,
                skill_id: Some(safe_name),
                action: "overwritten".to_string(),
            }
        }
        ConflictResolution::Coexist => {
            let coexist_name = request
                .coexist_name
                .unwrap_or_else(|| format!("{}-project", skill_name));
            let safe_name = sanitize_dir_name(&coexist_name);
            ResolveConflictResult {
                success: true,
                skill_id: Some(safe_name),
                action: "coexisted".to_string(),
            }
        }
    };

    Ok(result)
}

#[tauri::command]
pub fn create_skill_version(request: CreateVersionRequest) -> Result<CreateVersionResponse, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let manager_dir = home.join(".skills-manager/skills");
    let source_path = PathBuf::from(&request.source_path);

    if !source_path.exists() || !source_path.join("SKILL.md").exists() {
        return Err("Source skill path is invalid".to_string());
    }

    let package = get_skill_package(GetSkillPackageRequest {
        skill_id: request.skill_id.clone(),
    })?
    .package;

    let reference_version = package
        .versions
        .iter()
        .find(|version| version.id == package.default_version)
        .cloned()
        .or_else(|| package.versions.first().cloned())
        .ok_or_else(|| "Skill package has no base versions".to_string())?;

    let destination_dir_name = format!(
        "{}-{}",
        sanitize_dir_name(&package.name),
        sanitize_dir_name(&request.display_name)
    );
    let destination_path = manager_dir.join(destination_dir_name);

    if destination_path.exists() {
        return Err("A version with the same destination folder already exists".to_string());
    }

    copy_dir_recursive(&source_path, &destination_path).map_err(|err| err.to_string())?;

    let sidecar = StoredVersionMetadata {
        version: Some(request.version.clone()),
        display_name: Some(request.display_name.clone()),
        source_url: request.source_url.clone(),
        parent_version: request.parent_version.clone().or(Some(reference_version.id)),
        deleted: Some(false),
    };
    write_version_sidecar(&destination_path, &sidecar)?;

    let created_version = build_skill_version(&destination_path, request.source.clone());
    write_version_metadata(&home, &created_version)?;

    Ok(CreateVersionResponse {
        version: created_version,
        installed_path: destination_path.display().to_string(),
    })
}

#[tauri::command]
pub fn analyze_skill_conflict(request: AnalyzeConflictRequest) -> Result<ConflictAnalysis, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let manager_dir = home.join(".skills-manager/skills");
    let existing_skills = collect_skills_from_dir(&manager_dir, "manager", None);

    let base_skill = existing_skills
        .into_iter()
        .find(|skill| {
            skill.current_version.as_ref().is_some_and(|version| {
                version.skill_id == request.skill_id && version.id == request.base_version_id
            })
        })
        .ok_or_else(|| "Base version not found".to_string())?;

    let base_version = base_skill
        .current_version
        .ok_or_else(|| "Base version metadata is missing".to_string())?;
    let incoming_path = PathBuf::from(&request.incoming_path);
    let incoming_version = build_skill_version(&incoming_path, SkillVersionSource::Project);
    let diff = build_skill_diff(&base_version, &incoming_version);
    let (conflict_type, severity, auto_resolvable, suggestions) = classify_conflict(&diff);

    Ok(ConflictAnalysis {
        conflict_type,
        severity,
        base_version,
        incoming_version,
        diff,
        auto_resolvable,
        suggestions,
    })
}

#[tauri::command]
pub fn compare_skill_versions(request: crate::types::CompareVersionsRequest) -> Result<SkillDiff, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let manager_dir = home.join(".skills-manager/skills");
    let package = get_skill_package(GetSkillPackageRequest {
        skill_id: request.skill_id,
    })?
    .package;

    let from_version = package
        .versions
        .iter()
        .find(|version| version.id == request.from_version)
        .cloned()
        .ok_or_else(|| "Source version not found".to_string())?;

    let to_version = package
        .versions
        .iter()
        .find(|version| version.id == request.to_version)
        .cloned()
        .ok_or_else(|| "Target version not found".to_string())?;

    let _ = manager_dir;
    Ok(build_skill_diff(&from_version, &to_version))
}

#[tauri::command]
pub fn list_skill_packages() -> Result<ListSkillPackagesResponse, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let manager_dir = home.join(".skills-manager/skills");
    let mut packages = Vec::new();
    let mut seen: HashMap<String, bool> = HashMap::new();
    for skill in collect_skills_from_dir(&manager_dir, "manager", None) {
        if let Some(version) = skill.current_version {
            if seen.insert(version.skill_id.clone(), true).is_some() {
                continue;
            }
            let package = package_from_skill_dir(&home, &manager_dir, Path::new(&skill.path));
            if let Some(first) = package.versions.first() {
                packages.push(SkillPackageSummary {
                    id: package.id,
                    name: package.name,
                    namespace: package.namespace,
                    version_count: package.versions.len(),
                    variant_count: package.variants.len(),
                    latest_version: first.version.clone(),
                    default_version: package.default_version,
                    updated_at: package.updated_at,
                });
            }
        }
    }
    packages.sort_by(|left, right| left.name.cmp(&right.name));

    Ok(ListSkillPackagesResponse {
        total: packages.len(),
        packages,
    })
}

#[tauri::command]
pub fn get_skill_package(request: GetSkillPackageRequest) -> Result<GetSkillPackageResponse, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let manager_dir = home.join(".skills-manager/skills");

    let package = collect_skills_from_dir(&manager_dir, "manager", None)
        .into_iter()
        .find_map(|skill| {
            skill.current_version.as_ref().and_then(|version| {
                if version.skill_id == request.skill_id {
                    Some(package_from_skill_dir(&home, &manager_dir, Path::new(&skill.path)))
                } else {
                    None
                }
            })
        })
        .ok_or_else(|| "Skill package not found".to_string())?;

    Ok(GetSkillPackageResponse { package })
}

#[tauri::command]
pub fn rename_skill_version(
    request: RenameVersionRequest,
) -> Result<RenameVersionResponse, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let manager_dir = home.join(".skills-manager/skills");
    let skill = collect_skills_from_dir(&manager_dir, "manager", None)
        .into_iter()
        .find(|item| {
            item.current_version.as_ref().is_some_and(|version| {
                version.skill_id == request.skill_id && version.id == request.version_id
            })
        })
        .ok_or_else(|| "Version not found".to_string())?;

    let skill_path = PathBuf::from(&skill.path);
    let mut sidecar = read_version_sidecar(&skill_path);
    sidecar.display_name = Some(request.new_display_name.clone());
    let sidecar_path = skill_path.join(VERSION_METADATA_FILE);
    let serialized = serde_json::to_string_pretty(&sidecar).map_err(|err| err.to_string())?;
    fs::write(sidecar_path, serialized).map_err(|err| err.to_string())?;

    let version = version_summary_for_skill(&home, &skill_path);
    Ok(RenameVersionResponse {
        success: true,
        version,
    })
}

#[tauri::command]
pub fn delete_skill_version(
    request: DeleteVersionRequest,
) -> Result<DeleteVersionResponse, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let manager_dir = home.join(".skills-manager/skills");
    let skill = collect_skills_from_dir(&manager_dir, "manager", None)
        .into_iter()
        .find(|item| {
            item.current_version.as_ref().is_some_and(|version| {
                version.skill_id == request.skill_id && version.id == request.version_id
            })
        })
        .ok_or_else(|| "Version not found".to_string())?;

    let skill_path = PathBuf::from(&skill.path);
    let force = request.force.unwrap_or(false);
    let version_count = collect_skills_from_dir(&manager_dir, "manager", None)
        .into_iter()
        .filter(|item| {
            item.current_version
                .as_ref()
                .is_some_and(|version| version.skill_id == request.skill_id)
        })
        .count();

    if version_count <= 1 && !force {
        return Err("Refusing to delete the only available version without force".to_string());
    }

    match request.strategy {
        DeleteStrategy::Soft => {
            let mut sidecar = read_version_sidecar(&skill_path);
            sidecar.deleted = Some(true);
            let serialized = serde_json::to_string_pretty(&sidecar).map_err(|err| err.to_string())?;
            fs::write(skill_path.join(VERSION_METADATA_FILE), serialized).map_err(|err| err.to_string())?;
            Ok(DeleteVersionResponse {
                success: true,
                message: "Version marked as deleted".to_string(),
                archived_path: None,
            })
        }
        DeleteStrategy::Archive => {
            let archive_root = home.join(".skills-manager/archive");
            fs::create_dir_all(&archive_root).map_err(|err| err.to_string())?;
            let archive_path = archive_root.join(
                skill_path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or("skill-version"),
            );
            fs::rename(&skill_path, &archive_path).map_err(|err| err.to_string())?;
            Ok(DeleteVersionResponse {
                success: true,
                message: "Version archived".to_string(),
                archived_path: Some(archive_path.display().to_string()),
            })
        }
        DeleteStrategy::Hard => {
            fs::remove_dir_all(&skill_path).map_err(|err| err.to_string())?;
            Ok(DeleteVersionResponse {
                success: true,
                message: "Version deleted".to_string(),
                archived_path: None,
            })
        }
    }
}

#[tauri::command]
pub fn set_default_skill_version(
    request: SetDefaultVersionRequest,
) -> Result<SkillVersion, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let skill_id = request.skill_id.clone();
    let version_id = request.version_id.clone();
    let package = get_skill_package(GetSkillPackageRequest {
        skill_id,
    })?;
    let version = package
        .package
        .versions
        .into_iter()
        .find(|version| version.id == version_id)
        .ok_or_else(|| "Version not found".to_string())?;

    let mut state = read_package_state(&home, &version.skill_id);
    state.default_version = Some(request.version_id.clone());
    if let Some(default_variant) = state.variants.iter_mut().find(|variant| variant.name == "default") {
        default_variant.current_version = request.version_id;
    }
    write_package_state(&home, &version.skill_id, &state)?;

    Ok(version)
}

#[tauri::command]
pub fn create_skill_variant(
    request: CreateVariantRequest,
) -> Result<CreateVariantResponse, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let mut state = read_package_state(&home, &request.skill_id);
    let variant = SkillVariant {
        id: format!("{}-{}", request.skill_id, sanitize_dir_name(&request.name)),
        name: request.name,
        current_version: request.version_id,
        created_at: now_timestamp(),
        description: request.description,
    };
    state.variants.push(variant.clone());
    write_package_state(&home, &request.skill_id, &state)?;
    Ok(CreateVariantResponse { variant })
}

#[tauri::command]
pub fn update_skill_variant(request: UpdateVariantRequest) -> Result<SkillVariant, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let mut state = read_package_state(&home, &request.skill_id);
    let variant = state
        .variants
        .iter_mut()
        .find(|variant| variant.id == request.variant_id)
        .ok_or_else(|| "Variant not found".to_string())?;

    if let Some(new_name) = request.new_name {
        variant.name = new_name;
    }
    if let Some(new_version_id) = request.new_version_id {
        variant.current_version = new_version_id;
    }
    if request.new_description.is_some() {
        variant.description = request.new_description;
    }

    let updated = variant.clone();
    write_package_state(&home, &request.skill_id, &state)?;
    Ok(updated)
}

#[tauri::command]
pub fn delete_skill_variant(request: DeleteVariantRequest) -> Result<(), String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let mut state = read_package_state(&home, &request.skill_id);
    let before = state.variants.len();
    state.variants.retain(|variant| variant.id != request.variant_id);
    if before == state.variants.len() {
        return Err("Variant not found".to_string());
    }
    write_package_state(&home, &request.skill_id, &state)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{build_skill_diff, classify_conflict, parse_skill_metadata, simple_hash};
    use crate::types::{ConflictSeverity, ConflictType, ResolutionAction, SkillVersion, SkillVersionMetadata, SkillVersionSource};
    use std::fs;
    use std::path::PathBuf;

    fn fixture_version(description: &str, version: &str, hash: &str) -> SkillVersion {
        SkillVersion {
            id: format!("{}-{}", version, hash),
            skill_id: "demo_default".to_string(),
            version: version.to_string(),
            display_name: version.to_string(),
            content_hash: hash.to_string(),
            created_at: 0,
            source: SkillVersionSource::Migration,
            source_url: None,
            parent_version: None,
            is_active: true,
            metadata: SkillVersionMetadata {
                name: "Demo".to_string(),
                description: description.to_string(),
                author: Some("A".to_string()),
                namespace: Some("default".to_string()),
            },
        }
    }

    #[test]
    fn simple_hash_is_stable() {
        assert_eq!(simple_hash("abc"), simple_hash("abc"));
        assert_ne!(simple_hash("abc"), simple_hash("def"));
    }

    #[test]
    fn parse_skill_metadata_reads_version_fields() {
        let temp_dir = std::env::temp_dir().join("skills-manager-parse-skill-metadata");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).expect("create temp dir");
        fs::write(
            temp_dir.join("SKILL.md"),
            "---\nname: Demo\nversion: 2.1.0\nauthor: Tester\nnamespace: team\n---\nDescription line\n",
        )
        .expect("write skill file");

        let parsed = parse_skill_metadata(&PathBuf::from(&temp_dir));
        assert_eq!(parsed.name, "Demo");
        assert_eq!(parsed.version.as_deref(), Some("2.1.0"));
        assert_eq!(parsed.author.as_deref(), Some("Tester"));
        assert_eq!(parsed.namespace.as_deref(), Some("team"));
        assert_eq!(parsed.description, "Description line");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn classify_conflict_detects_identical_versions() {
        let base = fixture_version("same", "1.0.0", "aaaa");
        let incoming = fixture_version("same", "1.0.0", "aaaa");
        let diff = build_skill_diff(&base, &incoming);
        let (conflict_type, severity, auto_resolvable, suggestions) = classify_conflict(&diff);
        assert_eq!(conflict_type, ConflictType::Identical);
        assert_eq!(severity, ConflictSeverity::None);
        assert!(auto_resolvable);
        assert_eq!(suggestions[0].action, ResolutionAction::FastForward);
    }
}
