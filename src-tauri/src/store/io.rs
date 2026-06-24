use crate::error::{AppError, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::Write;
use std::path::Path;

/// 값을 JSON으로 직렬화해 원자적으로 쓴다(temp → rename). 부모 디렉터리는 자동 생성.
pub fn write_atomic<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|_| AppError::Invalid("디렉터리 생성에 실패했습니다".into()))?;
    }
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|_| AppError::Invalid("직렬화에 실패했습니다".into()))?;
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| AppError::Invalid("파일명이 유효하지 않습니다".into()))?;
    let tmp = path.with_file_name(format!(".{file_name}.tmp"));
    {
        let mut f = std::fs::File::create(&tmp)
            .map_err(|_| AppError::Invalid("임시 파일 생성에 실패했습니다".into()))?;
        f.write_all(&bytes)
            .map_err(|_| AppError::Invalid("파일 쓰기에 실패했습니다".into()))?;
        f.sync_all()
            .map_err(|_| AppError::Invalid("파일 동기화에 실패했습니다".into()))?;
    }
    std::fs::rename(&tmp, path)
        .map_err(|_| AppError::Invalid("파일 교체에 실패했습니다".into()))?;
    Ok(())
}

/// JSON 파일을 읽어 역직렬화. 파일 없음은 NotFound.
pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = std::fs::read(path).map_err(|_| AppError::NotFound)?;
    serde_json::from_slice(&bytes)
        .map_err(|_| AppError::Invalid("JSON 파싱에 실패했습니다".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Sample { id: String, n: i64 }

    fn tmp_dir() -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("store_io_{}", crate::store::ids::new_id()));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn write_then_read_roundtrip() {
        let dir = tmp_dir();
        let path = dir.join("a.json");
        let s = Sample { id: "x".into(), n: 7 };
        write_atomic(&path, &s).unwrap();
        let back: Sample = read_json(&path).unwrap();
        assert_eq!(back, s);
    }

    #[test]
    fn write_creates_missing_parent_dirs() {
        let dir = tmp_dir();
        let path = dir.join("deep/nested/a.json");
        write_atomic(&path, &Sample { id: "y".into(), n: 1 }).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn write_overwrites_and_leaves_no_tmp() {
        let dir = tmp_dir();
        let path = dir.join("a.json");
        write_atomic(&path, &Sample { id: "x".into(), n: 1 }).unwrap();
        write_atomic(&path, &Sample { id: "x".into(), n: 2 }).unwrap();
        let back: Sample = read_json(&path).unwrap();
        assert_eq!(back.n, 2);
        // 임시 파일이 남지 않아야 함
        let leftovers: Vec<_> = std::fs::read_dir(&dir).unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("tmp"))
            .collect();
        assert!(leftovers.is_empty());
    }

    #[test]
    fn read_missing_file_is_not_found() {
        let dir = tmp_dir();
        let r: Result<Sample> = read_json(&dir.join("nope.json"));
        assert!(matches!(r, Err(crate::error::AppError::NotFound)));
    }
}
