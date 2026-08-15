use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct ArchiveResult {
    pub src: String,
    pub dest: String,
    pub ok: bool,
    pub error: Option<String>,
}

/// 把文件移动到目标目录，同名自动追加序号，绝不覆盖。
pub fn archive_files(files: Vec<String>, dest_dir: &str) -> Result<Vec<ArchiveResult>, String> {
    let dest = Path::new(dest_dir);
    if !dest.is_dir() {
        return Err(format!("目标目录不存在: {dest_dir}"));
    }

    let mut results = Vec::new();
    for file in files {
        let src = Path::new(&file);
        let Some(file_name) = src.file_name().map(|s| s.to_string_lossy().to_string()) else {
            results.push(ArchiveResult {
                src: file.clone(),
                dest: String::new(),
                ok: false,
                error: Some("无法解析文件名".to_string()),
            });
            continue;
        };

        let target = unique_path(dest, &file_name);
        match std::fs::rename(src, &target) {
            Ok(_) => results.push(ArchiveResult {
                src: file,
                dest: target.to_string_lossy().to_string(),
                ok: true,
                error: None,
            }),
            Err(e) => results.push(ArchiveResult {
                src: file,
                dest: target.to_string_lossy().to_string(),
                ok: false,
                error: Some(e.to_string()),
            }),
        }
    }
    Ok(results)
}

/// 生成不冲突的目标路径：存在则追加 (1) (2)...
fn unique_path(dest: &Path, file_name: &str) -> PathBuf {
    let candidate = dest.join(file_name);
    if !candidate.exists() {
        return candidate;
    }
    let stem = Path::new(file_name)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());
    let ext = Path::new(file_name)
        .extension()
        .map(|s| s.to_string_lossy().to_string());

    let mut i = 1;
    loop {
        let new_name = match &ext {
            Some(e) => format!("{stem}({i}).{e}"),
            None => format!("{stem}({i})"),
        };
        let candidate = dest.join(&new_name);
        if !candidate.exists() {
            return candidate;
        }
        i += 1;
    }
}

/// 在系统资源管理器中打开目录（Windows），开发环境（Linux）用 xdg-open。
pub fn open_in_explorer(path: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
