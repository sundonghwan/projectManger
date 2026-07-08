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

/// 앱 Vault 루트 폴더 이름. 사용자가 고른 vault 안에 이 폴더를 만들어 그 아래로 정리한다.
pub const APP_VAULT_DIR: &str = "Work Vault";

/// 앱 Vault 루트: vaultPath 있으면 `<vaultPath>/Work Vault`, 없으면 `<appData>/Work Vault`.
pub fn app_vault_root(app_data_dir: &Path) -> PathBuf {
    match read_vault_path(app_data_dir) {
        Some(p) => PathBuf::from(p).join(APP_VAULT_DIR),
        None => app_data_dir.join(APP_VAULT_DIR),
    }
}

/// 실제 store 루트: `<app_vault_root>/.projectManger`.
pub fn store_root(app_data_dir: &Path) -> PathBuf {
    app_vault_root(app_data_dir).join(".projectManger")
}

/// 구 레이아웃(`<vaultPath>/.projectManger`)을 신 레이아웃(`<vaultPath>/Work Vault/.projectManger`)으로
/// 1회 이동한다. 같은 볼륨이면 rename 은 atomic. Store::open 이전에 호출해야 한다.
pub fn relocate_into_work_vault(app_data_dir: &Path) -> Result<()> {
    let Some(vault) = read_vault_path(app_data_dir) else {
        return Ok(()); // 기본 위치 사용 시 재배치 불필요
    };
    let vault = PathBuf::from(vault);
    let legacy = vault.join(".projectManger");
    let new_root = vault.join(APP_VAULT_DIR).join(".projectManger");
    if new_root.exists() || !legacy.exists() {
        return Ok(()); // 이미 이동됐거나 구 데이터 없음
    }
    std::fs::create_dir_all(vault.join(APP_VAULT_DIR))
        .map_err(|e| AppError::Invalid(format!("Work Vault 생성 실패: {e}")))?;
    std::fs::rename(&legacy, &new_root)
        .map_err(|e| AppError::Invalid(format!(".projectManger 이동 실패: {e}")))?;
    Ok(())
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
        assert_eq!(app_vault_root(&d), d.join("Work Vault"));
        assert_eq!(store_root(&d), d.join("Work Vault").join(".projectManger"));
    }

    #[test]
    fn set_then_read_and_root() {
        let d = tmp();
        let vault = tmp();
        write_vault_path(&d, Some(vault.to_str().unwrap())).unwrap();
        assert_eq!(read_vault_path(&d).as_deref(), vault.to_str());
        assert_eq!(app_vault_root(&d), vault.join("Work Vault"));
        assert_eq!(store_root(&d), vault.join("Work Vault").join(".projectManger"));
        // null 로 초기화하면 기본으로
        write_vault_path(&d, None).unwrap();
        assert_eq!(read_vault_path(&d), None);
        assert_eq!(store_root(&d), d.join("Work Vault").join(".projectManger"));
    }

    #[test]
    fn relocate_moves_legacy_projectmanger_into_work_vault() {
        let d = tmp();
        let vault = tmp();
        write_vault_path(&d, Some(vault.to_str().unwrap())).unwrap();
        // 구 위치 생성
        let legacy = vault.join(".projectManger");
        std::fs::create_dir_all(legacy.join("businesses")).unwrap();
        std::fs::write(legacy.join("businesses").join("x.json"), b"{}").unwrap();

        relocate_into_work_vault(&d).unwrap();

        assert!(!legacy.exists());
        let moved = vault.join("Work Vault").join(".projectManger").join("businesses").join("x.json");
        assert!(moved.is_file());
        // 멱등: 두 번째 호출은 무해
        relocate_into_work_vault(&d).unwrap();
        assert!(moved.is_file());
    }
}
