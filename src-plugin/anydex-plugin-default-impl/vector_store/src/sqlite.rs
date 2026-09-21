use std::sync::Mutex;

use rusqlite::Connection;

static DB: Mutex<Option<Connection>> = Mutex::new(None);

pub fn sqlite_init(sqlite_extension_path: &str) -> Result<(), String> {
    let conn = Connection::open("vector-store.db").map_err(|err| err.to_string())?;
    unsafe {
        conn.load_extension_enable()
            .map_err(|err| err.to_string())?;
    }

    // load vector extension
    conn.execute("SELECT load_extension(?)", [sqlite_extension_path])
        .map_err(|err| err.to_string())?;

    // create table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS chunks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT UNIQUE
        )",
        [],
    )
    .map_err(|err| err.to_string());

    let mut db_mtx = DB.lock().unwrap();
    (*db_mtx) = Some(conn);
    Ok(())
}
