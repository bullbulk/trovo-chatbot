use unqlite::{UnQLite, KV};

const DB_NAME: &str = "unqlite.db";


// Write value by key. Both should can be casted as u8 array
pub fn write<K: AsRef<[u8]>, V: AsRef<[u8]>>(key: K, value: V) {
    let unqlite = UnQLite::create(DB_NAME);
    unqlite.kv_store(key, value).unwrap();
}

// Get value by key. If not exists, returns empty Vec<u8>
pub fn read<K: AsRef<[u8]>>(key: K) -> Vec<u8> {
    let unqlite = UnQLite::create(DB_NAME);
    return match unqlite.kv_fetch(key) {
        Ok(i) => i,
        Err(_) => Vec::new(),
    };
}