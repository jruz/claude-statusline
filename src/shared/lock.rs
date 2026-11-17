use std::path::Path;

use crate::shared::logging;

const STALE_AFTER_SECONDS: u64 = 300;

pub enum State {
    Free,
    Held,
    Stale,
}

pub fn state(lock_path: &Path) -> State {
    if !lock_path.exists() {
        return State::Free;
    }
    if is_stale(lock_path) {
        return State::Stale;
    }
    State::Held
}

fn is_stale(lock_path: &Path) -> bool {
    let metadata = match std::fs::metadata(lock_path) {
        Ok(meta) => meta,
        Err(err) => {
            logging::log_error("lock", &format!("metadata error: {err}"));
            return true;
        }
    };
    let modified = match metadata.modified() {
        Ok(mtime) => mtime,
        Err(err) => {
            logging::log_error("lock", &format!("mtime error: {err}"));
            return true;
        }
    };
    let age = match std::time::SystemTime::now().duration_since(modified) {
        Ok(dur) => dur,
        Err(err) => {
            logging::log_error("lock", &format!("age error: {err}"));
            return true;
        }
    };
    age.as_secs() >= STALE_AFTER_SECONDS
}

#[cfg(test)]
mod tests {
    use super::*;

    fn age_file(path: &Path, seconds: i64) {
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time since epoch")
            .as_secs();
        let past_secs = i64::try_from(now_secs).expect("secs to i64") - seconds;
        let mtime = filetime::FileTime::from_unix_time(past_secs, 0);
        filetime::set_file_mtime(path, mtime).expect("set mtime");
    }

    #[test]
    fn state_is_free_without_lock() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let lock_path = dir.path().join("test.lock");

        assert!(matches!(state(&lock_path), State::Free));
    }

    #[test]
    fn state_is_held_for_fresh_lock() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let lock_path = dir.path().join("test.lock");
        std::fs::write(&lock_path, "").expect("write lock");

        assert!(matches!(state(&lock_path), State::Held));
    }

    #[test]
    fn state_is_stale_for_old_lock() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let lock_path = dir.path().join("test.lock");
        std::fs::write(&lock_path, "").expect("write lock");
        age_file(&lock_path, 600);

        assert!(matches!(state(&lock_path), State::Stale));
    }
}
