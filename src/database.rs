use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Deserialize, Serialize)]
pub struct FileListing {
    pub id: usize,
    pub path: String,
    pub password: Option<String>,
}

impl FileListing {
    pub fn from_path(path: &str, db: &Mutex<Connection>) -> Option<Self> {
        db.lock()
            .unwrap()
            .query_row(
                "SELECT id, path, password FROM files WHERE path=?1",
                params![path],
                |row| {
                    Ok(Self {
                        id: row.get_unwrap(0),
                        path: row.get_unwrap(1),
                        password: row.get_unwrap(2),
                    })
                },
            )
            .ok()
    }
    pub fn read_all(db: &Mutex<Connection>) -> Result<Vec<Self>, rusqlite::Error> {
        let binding = db.lock().unwrap();
        let mut stmt = binding.prepare("SELECT id, path, password FROM files")?;
        let mut rows = stmt.query([])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(FileListing {
                id: row.get_unwrap(0),
                path: row.get_unwrap(1),
                password: row.get_unwrap(2),
            });
        }
        Ok(out)
    }
    pub fn push_to_db(&self, db: &Mutex<Connection>) -> Result<usize, rusqlite::Error> {
        db.lock().unwrap().execute(
            "INSERT INTO files (path, password) VALUES (?1,?2)",
            params![self.path, self.password],
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkListing {
    pub id: usize,
    pub path: String,
    pub name: String,
    pub destination: String,
}

impl LinkListing {
    pub fn from_path(path: &str, db: &Mutex<Connection>) -> Option<Self> {
        db.lock()
            .unwrap()
            .query_row(
                "SELECT id, path, name, destination FROM links WHERE path=?1",
                params![path],
                |row| {
                    Ok(Self {
                        id: row.get_unwrap(0),
                        path: row.get_unwrap(1),
                        name: row.get_unwrap(2),
                        destination: row.get_unwrap(3),
                    })
                },
            )
            .ok()
    }
    pub fn read_all(db: &Mutex<Connection>) -> Result<Vec<Self>, rusqlite::Error> {
        let binding = db.lock().unwrap();
        let mut stmt = binding.prepare("SELECT id, path, name, destination FROM links")?;
        let mut rows = stmt.query([])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(LinkListing {
                id: row.get_unwrap(0),
                path: row.get_unwrap(1),
                name: row.get_unwrap(2),
                destination: row.get_unwrap(3),
            });
        }
        Ok(out)
    }
    pub fn push_to_db(&self, db: &Mutex<Connection>) -> Result<usize, rusqlite::Error> {
        db.lock().unwrap().execute(
            "INSERT INTO links (path, name, destination) VALUES (?1,?2,?3)",
            params![self.path, self.name, self.destination],
        )
    }
}

const CREATE_FILES_TABLE: &str = "
    CREATE TABLE IF NOT EXISTS files (
        id INTEGER PRIMARY KEY,
        path TEXT NOT NULL UNIQUE,
        password TEXT
    );";

const CREATE_LINKS_TABLE: &str = "
    CREATE TABLE IF NOT EXISTS links (
        id INTEGER PRIMARY KEY,
        path TEXT NOT NULL UNIQUE,
        name TEXT NOT NULL UNIQUE,
        destination TEXT NOT NULL
    );";

pub fn open_connection() -> Mutex<Connection> {
    let db = Mutex::new(Connection::open("./site.db").expect("Cannot open db"));
    db.lock().unwrap().execute(CREATE_FILES_TABLE, []).unwrap();
    db.lock().unwrap().execute(CREATE_LINKS_TABLE, []).unwrap();
    return db;
}
