use crate::error::{AppError, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};

fn config_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("config.json")
}

/// config.json 의 vaultPath(없으면 None).
pub fn read_vault_path(app_data_dir: &Path) -> Option<String> {
    let text = std::fs::read_to_string(config_path(app_data_dir)).ok()?;
    let v: Value = serde_json::from_str(&text).ok()?;
    v.get("vaultPath")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string())
}

/// vaultPath 기록(None 이면 기본으로 초기화).
pub fn write_vault_path(app_data_dir: &Path, vault_path: Option<&str>) -> Result<()> {
    std::fs::create_dir_all(app_data_dir).ok();
    let v = serde_json::json!({ "vaultPath": vault_path });
    let pretty = serde_json::to_string_pretty(&v)
        .map_err(|e| AppError::Invalid(format!("config 직렬화 오류: {e}")))?;
    std::fs::write(config_path(app_data_dir), pretty.as_bytes())
        .map_err(|e| AppError::Invalid(format!("config 쓰기 실패: {e}")))
}

/// 실제 store 루트: vaultPath 있으면 `<vaultPath>/.projectManger`, 없으면 `<appData>/.projectManger`.
pub fn store_root(app_data_dir: &Path) -> PathBuf {
    match read_vault_path(app_data_dir) {
        Some(p) => PathBuf::from(p).join(".projectManger"),
        None => app_data_dir.join(".projectManger"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp() -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("cfg_{}", crate::store::ids::new_id()));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn default_when_unset() {
        let d = tmp();
        assert_eq!(read_vault_path(&d), None);
        assert_eq!(store_root(&d), d.join(".projectManger"));
    }

    #[test]
    fn set_then_read_and_root() {
        let d = tmp();
        let vault = tmp();
        write_vault_path(&d, Some(vault.to_str().unwrap())).unwrap();
        assert_eq!(read_vault_path(&d).as_deref(), vault.to_str());
        assert_eq!(store_root(&d), vault.join(".projectManger"));
        // null 로 초기화하면 기본으로
        write_vault_path(&d, None).unwrap();
        assert_eq!(read_vault_path(&d), None);
        assert_eq!(store_root(&d), d.join(".projectManger"));
    }
}
