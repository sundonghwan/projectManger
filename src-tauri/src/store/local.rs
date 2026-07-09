use crate::error::Result;
use crate::store::collection::Collection;
use crate::store::model::{Server, Snippet};
use std::path::PathBuf;

/// SSH 등 동기화 제외 로컬 데이터 저장소(`<appData>/local/`).
pub struct LocalStore {
    pub root: PathBuf,
    pub servers: Collection<Server>,
    pub snippets: Collection<Snippet>,
}

impl LocalStore {
    pub fn open(root: PathBuf) -> Result<LocalStore> {
        let mut s = LocalStore {
            servers: Collection::new(&root),
            snippets: Collection::new(&root),
            root,
        };
        s.load()?;
        Ok(s)
    }

    pub fn load(&mut self) -> Result<()> {
        self.servers.load()?;
        self.snippets.load()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::ids::{new_id, now};
    use crate::store::model::{Server, Snippet};

    fn tmp() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("localstore_{}", new_id()))
    }

    #[test]
    fn open_empty_and_persist_across_reopen() {
        let root = tmp();
        {
            let mut ls = LocalStore::open(root.clone()).unwrap();
            assert_eq!(ls.servers.len(), 0);
            ls.servers.put(Server {
                id: "s1".into(), business_id: "b1".into(), project_id: None, name: "n".into(),
                host: "h".into(), port: 22, username: "u".into(), auth_type: "key".into(),
                key_path: None, secret_ref: None, last_used_at: None, ai_bridge: false, archived_at: None,
                created_at: now(), updated_at: now(),
            }).unwrap();
            ls.snippets.put(Snippet {
                id: "n1".into(), server_id: "s1".into(), name: "배포".into(), command: "c".into(),
                sort_order: 1.0, created_at: now(), updated_at: now(),
            }).unwrap();
        }
        let ls2 = LocalStore::open(root).unwrap();
        assert_eq!(ls2.servers.len(), 1);
        assert_eq!(ls2.snippets.get("n1").unwrap().server_id, "s1");
    }
}
