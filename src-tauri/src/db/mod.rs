use std::{collections::HashMap, fs::{self, File}, io::Read, path::{Path, PathBuf}, str::FromStr, sync::Mutex};
use serde_json::json;
use sqlx::{migrate, sqlite::{SqliteConnectOptions, SqlitePoolOptions}};
use tauri::{path::BaseDirectory, AppHandle, Manager as _};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons::YesNo};
use time::{Date, macros::date};

use crate::logic::Db;

pub struct AppData {
    pub directories: Directories,
    pub db_info: DbInfo,
}

impl AppData {
    fn build(directories: Directories, db_info: DbInfo) -> Self {
        Self {
            directories,
            db_info,
        }
    }
}

struct MiscData {
    today: Mutex<Date>,
}

impl Default for MiscData {
    fn default() -> Self {
        Self { today: Mutex::new(date!(2025-07-01)) }
    }
}

impl MiscData {
    // Get a JSON object.
    fn json(&self) -> serde_json::Value {
        json!({
            "today": *self.today.lock().unwrap()
        })
    }

    // Replace the existing data.
    fn overwrite(&self, json: serde_json::Value) {
        *self.today.lock().unwrap() = serde_json::from_value(json["today"].clone()).unwrap();
    }

    // Reset the object to its default values.
    fn default(&self) {
        *self.today.lock().unwrap() = date!(2025-07-01);
    }
}

pub struct Directories {
    names: PathBuf,
    flags: PathBuf,
    pub db: PathBuf,
}

pub struct DbInfo {
    pool: Db,
    pub path: Mutex<String>,   // The path where the database gets saved to and loaded from.
    pub in_sync: Mutex<bool>,  // Keeps track of unsaved changes in the database.
    data: MiscData,
    queries: HashMap<String, String>,
}

impl DbInfo {
    fn build(data: Db, path: String) -> Self {
        Self {
            pool: data,
            path: Mutex::new(path),
            in_sync: Mutex::new(true),
            data: Default::default(),
            queries: Self::initialise_queries(),
        }
    }

    fn initialise_queries() -> HashMap<String, String> {
        let mut queries = HashMap::new();

        let buf = PathBuf::from("migrations/more_sql/");
        let dir = fs::read_dir(&buf).unwrap();

        for entry in dir {
            // Get the filename.
            let filename = entry.unwrap().file_name().into_string().unwrap();
            if !filename.ends_with(".sql") { continue }
            let key = String::from(&filename[0..filename.len() - 4]);

            // Get the query.
            let mut sql = String::new();
            let mut file = File::open(format!("migrations/more_sql/{key}.sql")).unwrap();
            file.read_to_string(&mut sql).unwrap();

            // Add them together!
            queries.insert(key, sql);
        }

        return queries;
    }

    // Reset the database.
    pub async fn reset(&self) {
        self.defer_foreign_keys().await;

        let mut tx = self.pool.begin().await.unwrap();

        // Delete the non-static data from the database.
        sqlx::query(
            self.queries.get("delete_data").unwrap()
        ).execute(&mut *tx).await.unwrap();

        // Reset the runtime memory stuff.
        self.data.default();
    }

    // Create the database connection pool.
    pub async fn init_connection() -> Db {
        let options = SqliteConnectOptions::from_str("sqlite::memory:").unwrap();

        let data = SqlitePoolOptions::new()
            .connect_with(options).await.unwrap();

        migrate!("./migrations").run(&data).await.unwrap();
        return data;
    }

    // Build the default database file.
    async fn build_default(path: &Path) -> Self {
        let data = Self::init_connection().await;
        let info = Self::build(data, path.to_str().unwrap().to_string());
        info.save_to_file().await;
        return info;
    }

    // Load the default database.
    async fn load_default(url: &str) -> Self {
        let data = Self::init_connection().await;
        let info = Self::build(data, url.to_string());
        info.load_from_file().await;
        return info;
    }

    // Defer the foreign keys before interactions
    // between in-memory and on-disk databases.
    async fn defer_foreign_keys(&self) {
        sqlx::query(
            "PRAGMA defer_foreign_keys = TRUE"
        ).execute(&self.pool).await.unwrap();
    }

    // Save the database to file.
    pub async fn save_to_file(&self) {
        // Generate the blob from the database.
        let blob: Vec<u8> = sqlx::query_scalar(
            self.queries.get("save_data").unwrap()
        ).bind(self.data.json())
        .fetch_one(&self.pool).await.unwrap();

        // Write it on disk as bytes.
        fs::write(self.path.lock().unwrap().as_str(), &blob).unwrap();

        // The database is now synchronised.
        *self.in_sync.lock().unwrap() = true;
    }

    // Load the database from file.
    pub async fn load_from_file(&self) {
        // Load the blob from the database file.
        let blob = fs::read(self.path.lock().unwrap().as_str()).unwrap();

        self.defer_foreign_keys().await;

        let mut tx = self.pool.begin().await.unwrap();

        // Delete all previous data from the database.
        sqlx::query(
            self.queries.get("delete_data").unwrap()
        ).execute(&mut *tx).await.unwrap();

        // Insert the blob's data into the database.
        let json: serde_json::Value = sqlx::query_scalar(
            self.queries.get("insert_data").unwrap()
        ).bind(&blob)
        .fetch_one(&mut *tx).await.unwrap();

        tx.commit().await.unwrap();

        // Overwrite the in-memory data.
        self.data.overwrite(json);

        // The database is now synchronised.
        *self.in_sync.lock().unwrap() = true;
    }

    // A pop-up dialog that reminds the user about unsaved changes.
    pub async fn remind_save_dialog(&self, handle: &AppHandle) {
        if *self.in_sync.lock().unwrap() { return; }
        let save_changes = handle.dialog()
            .message("You have unsaved changes. Do you want to save them before continuing?")
            .title("Unsaved Changes")
            .buttons(YesNo)
            .blocking_show();

        if !save_changes { return; }
        self.save_to_file().await;
    }
}

impl Directories {
    fn build(data_dir: PathBuf) -> Self {
        let dirs = Self {
            names: data_dir.join("names/"),
            flags: data_dir.join("flags/"),
            db: data_dir.join("db/"),
        };

        return dirs;
    }
}

// Load the default database.
pub async fn initialise(handle: &AppHandle) -> AppData {
    #[cfg(debug_assertions)] {
        let path = Path::new("data/db/default.db");
        if !path.exists() {
            DbInfo::build_default(path).await;
            panic!("default database created, program must terminate");
        }
    }

    let data_dir = handle
        .path()
        .resolve("data/", BaseDirectory::Resource)
        .unwrap();

    let directories = Directories::build(data_dir);
    let default_db = directories.db.join("default.db");

    let db_info = DbInfo::load_default(default_db.to_str().unwrap()).await;
    return AppData::build(directories, db_info);
}