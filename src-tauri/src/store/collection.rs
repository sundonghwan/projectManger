use crate::error::{AppError, Result};
use crate::store::entity::Entity;
use crate::store::io;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 한 엔티티 타입의 인메모리 맵 + 디스크 write-through.
/// 레이아웃(이 단계): `<root>/<collection>/<id>.json` (플랫).
pub struct Collection<T: Entity> {
    dir: PathBuf,
    items: HashMap<String, T>,
}

impl<T: Entity> Collection<T> {
    pub fn new(root: &Path) -> Self {
        Collection {
            dir: root.join(T::collection()),
            items: HashMap::new(),
        }
    }

    fn path_for(&self, id: &str) -> PathBuf {
        self.dir.join(format!("{id}.json"))
    }

    /// 디스크에서 전량 로드. 손상되거나 읽을 수 없는 항목은 건너뛴다.
    /// 로컬 맵에 모은 뒤 성공 시에만 교체하므로, 디렉터리 읽기 실패 시
    /// 기존 인메모리 데이터를 잃지 않는다.
    pub fn load(&mut self) -> Result<()> {
        let mut loaded: HashMap<String, T> = HashMap::new();
        if self.dir.exists() {
            let rd = std::fs::read_dir(&self.dir)
                .map_err(|_| AppError::Invalid("디렉터리 읽기에 실패했습니다".into()))?;
            for entry in rd {
                let Ok(entry) = entry else { continue };
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Ok(item) = io::read_json::<T>(&path) {
                        loaded.insert(item.id().to_string(), item);
                    }
                }
            }
        }
        self.items = loaded;
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&T> {
        self.items.get(id)
    }

    pub fn list(&self) -> Vec<T> {
        self.items.values().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// 삽입/갱신(write-through).
    pub fn put(&mut self, item: T) -> Result<()> {
        io::write_atomic(&self.path_for(item.id()), &item)?;
        self.items.insert(item.id().to_string(), item);
        Ok(())
    }

    /// 영구 삭제(파일 + 인메모리).
    pub fn remove(&mut self, id: &str) -> Result<()> {
        let p = self.path_for(id);
        if p.exists() {
            std::fs::remove_file(&p)
                .map_err(|_| AppError::Invalid("파일 삭제에 실패했습니다".into()))?;
        }
        self.items.remove(id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::entity::Entity;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
    struct Widget { id: String, name: String, updated_at: String }

    impl Entity for Widget {
        fn collection() -> &'static str { "widgets" }
        fn id(&self) -> &str { &self.id }
        fn updated_at(&self) -> &str { &self.updated_at }
    }

    fn tmp_root() -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("store_col_{}", crate::store::ids::new_id()));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn w(id: &str, name: &str) -> Widget {
        Widget { id: id.into(), name: name.into(), updated_at: crate::store::ids::now() }
    }

    #[test]
    fn put_get_list_remove() {
        let root = tmp_root();
        let mut c: Collection<Widget> = Collection::new(&root);
        c.put(w("a", "A")).unwrap();
        c.put(w("b", "B")).unwrap();
        assert_eq!(c.len(), 2);
        assert_eq!(c.get("a").unwrap().name, "A");
        c.remove("a").unwrap();
        assert!(c.get("a").is_none());
        assert_eq!(c.len(), 1);
    }

    #[test]
    fn put_persists_to_disk_and_reloads() {
        let root = tmp_root();
        {
            let mut c: Collection<Widget> = Collection::new(&root);
            c.put(w("a", "A")).unwrap();
            c.put(w("b", "B")).unwrap();
        }
        let mut c2: Collection<Widget> = Collection::new(&root);
        c2.load().unwrap();
        assert_eq!(c2.len(), 2);
        assert_eq!(c2.get("b").unwrap().name, "B");
    }

    #[test]
    fn remove_deletes_file() {
        let root = tmp_root();
        let mut c: Collection<Widget> = Collection::new(&root);
        c.put(w("a", "A")).unwrap();
        c.remove("a").unwrap();
        let mut c2: Collection<Widget> = Collection::new(&root);
        c2.load().unwrap();
        assert_eq!(c2.len(), 0);
    }

    #[test]
    fn load_on_empty_dir_is_ok() {
        let root = tmp_root();
        let mut c: Collection<Widget> = Collection::new(&root);
        assert!(c.load().is_ok());
        assert_eq!(c.len(), 0);
    }
}
