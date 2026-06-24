use crate::error::{AppError, Result};
use crate::store::ids::{new_id, now};
use crate::store::local::LocalStore;
use crate::store::model::Snippet;
use crate::store::ops::util::cmp_sort;

pub fn list_by_server(local: &LocalStore, server_id: &str) -> Result<Vec<Snippet>> {
    let mut out: Vec<Snippet> = local
        .snippets
        .list()
        .into_iter()
        .filter(|s| s.server_id == server_id)
        .collect();
    out.sort_by(|a, b| cmp_sort(a.sort_order, &a.id, b.sort_order, &b.id));
    Ok(out)
}

fn next_sort(local: &LocalStore, server_id: &str) -> f64 {
    local
        .snippets
        .list()
        .iter()
        .filter(|s| s.server_id == server_id)
        .map(|s| s.sort_order)
        .fold(0.0_f64, f64::max)
        + 1.0
}

pub fn create(local: &mut LocalStore, server_id: &str, name: &str, command: &str) -> Result<Snippet> {
    if name.trim().is_empty() || command.trim().is_empty() {
        return Err(AppError::Invalid("이름과 명령은 필수입니다".into()));
    }
    let sort_order = next_sort(local, server_id);
    let ts = now();
    let s = Snippet {
        id: new_id(),
        server_id: server_id.to_string(),
        name: name.to_string(),
        command: command.to_string(),
        sort_order,
        created_at: ts.clone(),
        updated_at: ts,
    };
    local.snippets.put(s.clone())?;
    Ok(s)
}

pub fn delete(local: &mut LocalStore, id: &str) -> Result<()> {
    if local.snippets.get(id).is_none() {
        return Err(AppError::NotFound);
    }
    local.snippets.remove(id)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::ids::new_id;
    use crate::store::local::LocalStore;

    fn ls() -> LocalStore {
        LocalStore::open(std::env::temp_dir().join(format!("ops_snip_{}", new_id()))).unwrap()
    }

    #[test]
    fn create_list_delete() {
        let mut s = ls();
        let a = create(&mut s, "srv1", "배포", "./deploy.sh").unwrap();
        let b = create(&mut s, "srv1", "재시작", "./restart.sh").unwrap();
        assert!(a.sort_order < b.sort_order);
        assert_eq!(list_by_server(&s, "srv1").unwrap().len(), 2);
        delete(&mut s, &a.id).unwrap();
        assert_eq!(list_by_server(&s, "srv1").unwrap().len(), 1);
        assert!(matches!(delete(&mut s, "nope"), Err(AppError::NotFound)));
    }

    #[test]
    fn create_validates_and_scopes_by_server() {
        let mut s = ls();
        assert!(create(&mut s, "srv1", "", "x").is_err());
        assert!(create(&mut s, "srv1", "n", "  ").is_err());
        create(&mut s, "srv1", "a", "c").unwrap();
        create(&mut s, "srv2", "b", "c").unwrap();
        assert_eq!(list_by_server(&s, "srv1").unwrap().len(), 1);
        assert_eq!(list_by_server(&s, "srv2").unwrap().len(), 1);
    }
}
