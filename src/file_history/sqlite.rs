use std::{
    fs::{self}, os::unix::fs::MetadataExt, path::{Path, PathBuf}, str::FromStr,
};

use crate::file_history::util::{digest_file};

use super::util::FileUploadHistory;
use jiff::{tz::TimeZone, Timestamp, ZonedDateTime};
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;

#[derive(Clone, Debug)]
pub struct SqliteFileUploadHistory {
    pool: Pool<SqliteConnectionManager>,
}

impl SqliteFileUploadHistory {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        let conn = pool.get().unwrap();
        apply_migrations(&conn);
        return Self { pool };
    }

    fn existing_record(&self, path: &Path) -> Option<SqliteFileUploadRecord> {
        let conn = self.pool.get().unwrap();
        let result = conn
            .query_row(
                "SELECT id, path, last_modified, size, digest, uploaded
                FROM files
                WHERE path = ?1 AND uploaded = TRUE
            ",
                (path.to_str().unwrap(),),
                |r| {
                    let path_str: String = r.get(1).unwrap();
                    Ok(SqliteFileUploadRecord {
                        id: r.get::<usize, i64>(0).unwrap().try_into().unwrap(),
                        path: PathBuf::from_str(&path_str).unwrap(),
                        last_modified: r.get::<usize, String>(2).unwrap().parse().unwrap(),
                        size: r.get::<usize, i64>(3).unwrap().try_into().unwrap(),
                        digest: r.get(4).unwrap(),
                        uploaded: r.get(5).unwrap(),
                    })
                },
            )
            .ok();
        result
    }

    fn register_file(
        &self,
        path: &Path,
        modified: ZonedDateTime,
        size: u64,
    ) -> u64 {
        let digest = digest_file::<sha2::Sha256>(path);
        let conn = self.pool.get().unwrap();
        let result = conn
            .query_one(
                "INSERT INTO files (id, path, last_modified, size, digest, uploaded)
                VALUES (NULL, ?1, ?2, ?3, ?4, FALSE)
                ON CONFLICT (path)
                DO UPDATE SET last_modified = ?2, size =?3, digest = ?4,  uploaded = FALSE
                RETURNING id",
                (
                    path.to_str().unwrap(),
                    modified.to_string(),
                    i64::try_from(size).unwrap(),
                    digest,
                ),
                |x| x.get::<usize, i64>(0),
            )
            .unwrap()
            .try_into()
            .unwrap();
        result
    }

    fn confirm_upload(
        &self, id: u64
    ) {
        let conn = self.pool.get().unwrap();
        conn.execute("UPDATE files SET uploaded = TRUE WHERE id = ?1", (i64::try_from(id).unwrap(), )).unwrap();
    }
}

impl FileUploadHistory for SqliteFileUploadHistory {
    type ID = u64;
    fn check(&self, path: &Path) -> Option<Self::ID> {
        let record = self.existing_record(path);

        let metadata = fs::metadata(path).unwrap();
        let modified = metadata.modified().unwrap();
        let modified = Timestamp::try_from(modified)
            .unwrap()
            .to_zoned_datetime(TimeZone::system());

        match record {
            Some(record) => {
                if record.last_modified != modified || record.size != metadata.size() {
                    return Some(record.id);
                }
                return None;
            }
            None => {
                let id = self.register_file(path, modified, metadata.size());
                Some(id)
            }
        }
    }

    fn on_upload(&self, id: &Self::ID) {
        self.confirm_upload(*id);
    }
}

pub struct SqliteFileUploadRecord {
    id: u64,
    path: PathBuf,
    last_modified: ZonedDateTime,
    size: u64,
    digest: String,
    uploaded: bool,
}

pub fn apply_migrations(conn: &PooledConnection<SqliteConnectionManager>) -> u32 {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS migrations (version INTEGER)",
        (),
    )
    .unwrap();

    let db_version: u32 = conn
        .query_one("SELECT IFNULL(MAX(version), 0) FROM migrations", (), |x| x.get(0))
        .unwrap();

    let migrations = vec![
        (1, "CREATE TABLE files(
            id INTEGER PRIMARY KEY,
            path TEXT UNIQUE NOT NULL,
            last_modified TEXT NOT NULL,
            size INTEGER NOT NULL,
            digest TEXT NOT NULL,
            uploaded BOOL
        )")
    ];

    let mut change_count = 0;

    for (version, migration) in migrations {
        if db_version < version {
            println!("{:?}", version);
            conn.execute(migration, ()).unwrap();
            conn.execute("INSERT INTO migrations(version) VALUES (?1)", (version,)).unwrap();
            change_count += 1;
        }
    }
    
    change_count
}
