use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct FolderEntry {
    pub path: String,
    pub name: String,
    pub depth: usize,
    pub is_favorite: bool,
    pub exists: bool,
}

/// 扫描根目录下的子文件夹（递归，受 depth 限制），返回展平列表。
/// 隐藏目录（以 . 开头）跳过。
pub fn scan_roots(roots: &[String], favorites: &[String], max_depth: usize) -> Vec<FolderEntry> {
    let mut out: Vec<FolderEntry> = Vec::new();
    let fav_set: std::collections::HashSet<String> = favorites.iter().cloned().collect();

    for root in roots {
        let root_path = Path::new(root);
        if !root_path.is_dir() {
            out.push(FolderEntry {
                path: root.clone(),
                name: root_path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| root.clone()),
                depth: 0,
                is_favorite: fav_set.contains(root),
                exists: false,
            });
            continue;
        }
        walk(root_path, root_path, 0, max_depth, &fav_set, &mut out);
    }

    // 排序：深度优先、名称排序
    out.sort_by(|a, b| {
        a.depth
            .cmp(&b.depth)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    out
}

fn walk(
    root: &Path,
    dir: &Path,
    depth: usize,
    max_depth: usize,
    fav_set: &std::collections::HashSet<String>,
    out: &mut Vec<FolderEntry>,
) {
    let path_str = dir.to_string_lossy().to_string();
    let rel_name = if dir == root {
        root.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path_str.clone())
    } else {
        dir.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path_str.clone())
    };

    out.push(FolderEntry {
        exists: true,
        path: path_str.clone(),
        name: rel_name,
        depth,
        is_favorite: fav_set.contains(&path_str),
    });

    if depth >= max_depth {
        return;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let mut subdirs: Vec<std::path::PathBuf> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(file_name) = path.file_name() else { continue };
        let fname = file_name.to_string_lossy();
        if fname.starts_with('.') {
            continue;
        }
        if !path.is_dir() {
            continue;
        }
        subdirs.push(path);
    }
    subdirs.sort_by(|a, b| {
        a.file_name()
            .map(|s| s.to_string_lossy().to_lowercase())
            .cmp(&b.file_name().map(|s| s.to_string_lossy().to_lowercase()))
    });

    for sub in subdirs {
        walk(root, &sub, depth + 1, max_depth, fav_set, out);
    }
}
