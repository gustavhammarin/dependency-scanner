CREATE TABLE IF NOT EXISTS packages(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cache_path TEXT NOT NULL,
    package_id TEXT NOT NULL,
    version TEXT NOT NULL,
    package_source TEXT NOT NULL,
    fetch_date TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    UNIQUE(package_id, version, package_source)
);
