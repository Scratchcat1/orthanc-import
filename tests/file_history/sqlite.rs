#[cfg(test)]
mod test {
    use std::{fs::File, io::Write, time::{Duration, SystemTime}};

use r2d2::Pool;
    use r2d2_sqlite::SqliteConnectionManager;
    use tempfile::{TempDir};
    use orthanc_import::file_history::{sqlite::{SqliteFileUploadHistory, apply_migrations}, util::FileUploadHistory};


    fn get_pool(tempdir: &TempDir) -> Pool<SqliteConnectionManager> {
        let db_path = tempdir.path().join("sqlite.db");
        File::create(&db_path).unwrap();
        let manager = SqliteConnectionManager::file(&db_path);
        println!("{:?}, ",db_path.exists());
        let pool = r2d2::Pool::new(manager).unwrap();
        pool
    }

    #[test]
    fn apply_migrations_works() {
        let tempdir = TempDir::new().unwrap();
        let pool = get_pool(&tempdir);
        let changes = apply_migrations(&pool.get().unwrap());
        assert_eq!(changes, 1);
    }

    #[test]
    fn apply_migrations_only_applies_once() {
        let tempdir = TempDir::new().unwrap();
        let pool = get_pool(&tempdir);
        let changes = apply_migrations(&pool.get().unwrap());
        assert_eq!(changes, 1);

        let changes = apply_migrations(&pool.get().unwrap());
        assert_eq!(changes, 0);
    }

    #[test]
    fn file_changes_return_id() {
        let tempdir = TempDir::new().unwrap();
        let pool = get_pool(&tempdir);
        let history = SqliteFileUploadHistory::new(pool);

        let filepath = tempdir.path().join("testfile");
        let mut f = File::create(&filepath).unwrap();

        // Initial check returns an id
        let id = history.check(&filepath);
        assert_ne!(id, None);
        history.on_upload(&id.unwrap());

        // Unmodified returns None
        let id = history.check(&filepath);
        assert_eq!(id, None);

        // Modified returns an id again
        f.write(&[1,2,3,4]).unwrap();
        let id = history.check(&filepath);
        assert_ne!(id, None);
    }

    #[test]
    fn repeated_checks_return_id() {
        let tempdir = TempDir::new().unwrap();
        let pool = get_pool(&tempdir);
        let history = SqliteFileUploadHistory::new(pool);

        let filepath = tempdir.path().join("testfile");
        File::create(&filepath).unwrap();

        let id_1 = history.check(&filepath);
        let id_2 = history.check(&filepath);
        assert_eq!(id_1,id_2);
    }

    #[test]
    fn changed_modified_time_returns_id() {
        let tempdir = TempDir::new().unwrap();
        let pool = get_pool(&tempdir);
        let history = SqliteFileUploadHistory::new(pool);

        let filepath = tempdir.path().join("testfile");
        let f = File::create(&filepath).unwrap();

        // Add and mark as uploaded
        let id= history.check(&filepath);
        history.on_upload(&id.unwrap());

        // Change time
        f.set_modified(SystemTime::now().checked_add(Duration::from_millis(10)).unwrap()).unwrap();

        let id = history.check(&filepath);
        assert_ne!(id, None);
    }
}