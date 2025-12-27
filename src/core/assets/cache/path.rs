use std::path::{Path, PathBuf};

fn normalize_separators(path: &Path) -> PathBuf {
    let base = path.to_path_buf();

    #[cfg(unix)]
    {
        match base.to_str() {
            Some(s) if s.contains('\\') => PathBuf::from(s.replace('\\', "/")),
            _ => base,
        }
    }

    #[cfg(not(unix))]
    {
        base
    }
}

fn lexical_normalize(path: &Path) -> PathBuf {
    use std::path::Component;

    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                // Only pop if we have something real to pop.
                // Avoid popping past prefixes/root.
                let popped = normalized.pop();
                if !popped {
                    normalized.push(Component::ParentDir.as_os_str());
                }
            }
            other => normalized.push(other.as_os_str()),
        }
    }

    normalized
}

fn to_slash_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[derive(Debug, Clone)]
pub(crate) struct AssetPathInfo {
    pub(crate) key: String,
    pub(crate) io_path: PathBuf,
    pub(crate) is_portable: bool,
    pub(crate) reason: Option<&'static str>,
}

/// Analyze an input path against an `asset_root`.
///
/// This function does *not* log or enforce policy; it only computes a stable cache key
/// and a best-effort I/O path, plus whether the result is considered portable.
pub(crate) fn compute_asset_path_info(asset_root: &Path, input: &Path) -> AssetPathInfo {
    let input = normalize_separators(input);
    let root = lexical_normalize(&normalize_separators(asset_root));

    let input_was_absolute = input.is_absolute();

    // Resolve to an absolute-ish path for comparison/strip-prefix.
    let resolved = if input_was_absolute {
        input.clone()
    } else {
        root.join(&input)
    };

    let root_canon = std::fs::canonicalize(&root).unwrap_or_else(|_| root.clone());
    let resolved_lex = lexical_normalize(&resolved);
    let resolved_canon = std::fs::canonicalize(&resolved).unwrap_or_else(|_| resolved_lex.clone());

    let (relative_for_key, is_portable, reason) =
        if let Ok(rel) = resolved_canon.strip_prefix(&root_canon) {
            // Even if the resolved path is under asset_root on this machine, an *absolute input*
            // is usually not portable across machines. We still use the relative path as the key
            // to keep caching stable.
            let reason = if input_was_absolute {
                Some("absolute input path is not portable")
            } else {
                None
            };
            (rel.to_path_buf(), !input_was_absolute, reason)
        } else if let Ok(rel) = resolved_lex.strip_prefix(&root) {
            let reason = if input_was_absolute {
                Some("absolute input path is not portable")
            } else {
                None
            };
            (rel.to_path_buf(), !input_was_absolute, reason)
        } else if input_was_absolute {
            (
                resolved_canon.clone(),
                false,
                Some("absolute path outside asset_root"),
            )
        } else {
            // Relative input that escapes the asset root (often via `..`).
            (
                resolved_canon.clone(),
                false,
                Some("relative path escapes asset_root"),
            )
        };

    let mut key = to_slash_string(&relative_for_key);
    while key.starts_with("./") {
        key = key.trim_start_matches("./").to_string();
    }

    #[cfg(windows)]
    {
        key = key.to_lowercase();
    }

    AssetPathInfo {
        key,
        io_path: resolved_canon,
        is_portable,
        reason,
    }
}

/// Normalize an asset path into a stable cache key.
///
/// Goal: any input that points to the same file under `asset_root` produces the same key,
/// regardless of OS separators (`/` vs `\\`), redundant `.`/`..`, or absolute-vs-relative.
///
/// Notes:
/// - The returned key is a *virtual* path using `/` separators.
/// - When possible, the key is made relative to `asset_root`.
/// - On Windows, the key is lowercased to match the common case-insensitive filesystem.
pub(crate) fn normalize_asset_key(asset_root: &Path, input: &Path) -> String {
    compute_asset_path_info(asset_root, input).key
}

/// Resolve an asset path into a filesystem path for I/O.
///
/// This does not need to be stable across machines; it only needs to open the right file.
pub(crate) fn resolve_asset_path(asset_root: &Path, input: &Path) -> PathBuf {
    compute_asset_path_info(asset_root, input).io_path
}

#[cfg(test)]
mod tests {
    use super::{compute_asset_path_info, normalize_asset_key, resolve_asset_path};
    use std::path::Path;

    #[test]
    fn key_normalizes_separators_and_dot_segments() {
        let root = Path::new("/game");
        let k1 = normalize_asset_key(root, Path::new("assets\\images\\..\\img.png"));
        let k2 = normalize_asset_key(root, Path::new("./assets/img.png"));
        assert_eq!(k1, "assets/img.png");
        assert_eq!(k1, k2);
    }

    #[test]
    fn resolves_relative_paths_under_root() {
        let root = Path::new("/game");
        let resolved = resolve_asset_path(root, Path::new("assets/img.png"));
        // We only care that the path is joined correctly without panicking.
        // Canonicalization may fail in tests if the file doesn't exist.
        assert!(resolved.to_string_lossy().contains("assets"));
        assert!(
            resolved
                .to_string_lossy()
                .replace('\\', "/")
                .ends_with("/assets/img.png")
        );
    }

    #[cfg(windows)]
    #[test]
    fn key_lowercases_on_windows() {
        let root = Path::new(r"C:\\Game");
        let k = normalize_asset_key(root, Path::new(r"ASSETS\\IMG.PNG"));
        assert_eq!(k, "assets/img.png");
    }

    #[test]
    fn key_does_not_keep_parent_dir_segments_when_escaping_root() {
        let root = Path::new("/game");
        let k = normalize_asset_key(root, Path::new("../secret.txt"));
        assert!(!k.contains(".."));
        let k_slash = k.replace('\\', "/");
        assert!(k_slash.ends_with("/secret.txt") || k_slash.ends_with("secret.txt"));
    }

    #[test]
    fn info_reports_non_portable_escape() {
        let root = Path::new("/game");
        let info = compute_asset_path_info(root, Path::new("../secret.txt"));
        assert!(!info.is_portable);
        assert!(info.reason.is_some());
    }
}
