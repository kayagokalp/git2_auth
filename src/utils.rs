#[cfg(test)]
pub(crate) mod tests_utils {
    use std::{panic, path::Path};

    use home::home_dir;

    /// Create a tmp folder and execute the given test function `f`
    pub(crate) fn with_tmp_dir<F>(f: F)
    where
        F: FnOnce(&Path) + panic::UnwindSafe,
    {
        let tmp_dir_name = format!("git2-auth-test-{:x}", rand::random::<u64>());
        let home_dir = home_dir().unwrap();
        let tmp_dir = home_dir.join(".tmp").join(tmp_dir_name);
        std::fs::create_dir_all(&tmp_dir).unwrap();
        let panic = panic::catch_unwind(|| f(&tmp_dir));
        std::fs::remove_dir_all(&tmp_dir).unwrap();
        if let Err(e) = panic {
            panic::resume_unwind(e);
        }
    }
}
