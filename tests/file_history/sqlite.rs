#[cfg(test)]
mod test {
    use anyhow::{Ok, Result};
    use std::{
        fs::File,
        io::Write,
        time::{Duration, SystemTime},
    };

    use orthanc_import::file_history::{
        sqlite::{apply_migrations, SqliteFileUploadHistory},
        util::FileUploadHistory,
    };
    use r2d2::Pool;
    use r2d2_sqlite::SqliteConnectionManager;
    use tempfile::TempDir;

    fn get_pool(tempdir: &TempDir) -> Result<Pool<SqliteConnectionManager>> {
        let db_path = tempdir.path().join("sqlite.db");
        File::create(&db_path)?;
        let manager = SqliteConnectionManager::file(&db_path);
        println!("{:?}, ", db_path.exists());
        let pool = r2d2::Pool::new(manager)?;
        Ok(pool)
    }

    #[test]
    fn apply_migrations_works() -> Result<()> {
        let tempdir = TempDir::new()?;
        let pool = get_pool(&tempdir)?;
        let changes = apply_migrations(&pool.get()?);
        assert_eq!(changes, 1);
        Ok(())
    }

    #[test]
    fn apply_migrations_only_applies_once() -> Result<()> {
        let tempdir = TempDir::new()?;
        let pool = get_pool(&tempdir)?;
        let changes = apply_migrations(&pool.get()?);
        assert_eq!(changes, 1);

        let changes = apply_migrations(&pool.get()?);
        assert_eq!(changes, 0);
        Ok(())
    }

    #[test]
    fn file_changes_return_id() -> Result<()> {
        let tempdir = TempDir::new().unwrap();
        let pool = get_pool(&tempdir)?;
        let history = SqliteFileUploadHistory::new(pool);

        let filepath = tempdir.path().join("testfile");
        let mut f = File::create(&filepath)?;

        // Initial check returns an id
        let id = history.check(&filepath);
        assert_ne!(id, None);
        history.on_upload(&id.unwrap());

        // Unmodified returns None
        let id = history.check(&filepath);
        assert_eq!(id, None);

        // Modified returns an id again
        f.write(&[1, 2, 3, 4]).unwrap();
        let id = history.check(&filepath);
        assert_ne!(id, None);
        Ok(())
    }

    #[test]
    fn repeated_checks_return_id() -> Result<()> {
        let tempdir = TempDir::new()?;
        let pool = get_pool(&tempdir)?;
        let history = SqliteFileUploadHistory::new(pool);

        let filepath = tempdir.path().join("testfile");
        File::create(&filepath)?;

        let id_1 = history.check(&filepath);
        let id_2 = history.check(&filepath);
        assert_eq!(id_1, id_2);
        Ok(())
    }

    #[test]
    fn changed_modified_time_returns_id() -> Result<()> {
        let tempdir = TempDir::new()?;
        let pool = get_pool(&tempdir)?;
        let history = SqliteFileUploadHistory::new(pool);

        let filepath = tempdir.path().join("testfile");
        let f = File::create(&filepath)?;

        // Add and mark as uploaded
        let id = history.check(&filepath);
        history.on_upload(&id.unwrap());

        // Change time
        f.set_modified(
            SystemTime::now()
                .checked_add(Duration::from_millis(10))
                .unwrap(),
        )?;

        let id = history.check(&filepath);
        assert_ne!(id, None);
        Ok(())
    }
}
