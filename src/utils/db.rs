use kv::{Config, Store};

const DB_NAME: &str = "data";


// Write value by key. Both should can be casted as u8 array
pub fn write<'a, T>(key: T, value: T)
    where T: kv::Key<'a>, T: kv::Value
{
    let cfg = Config::new(DB_NAME);
    let store = Store::new(cfg).unwrap();
    let bucket = store.bucket::<T, T>(Some("store")).unwrap();
    bucket.set(&key, &value).unwrap();
}

// Get value by key. If not exists, returns empty Vec<u8>
pub fn read<'a, T>(key: T) -> Option<T>
    where T: kv::Key<'a>, T: kv::Value
{
    let cfg = Config::new(DB_NAME);
    let store = Store::new(cfg).unwrap();
    let bucket = store.bucket::<T, T>(Some("store")).unwrap();
    return bucket.get(&key).unwrap();
}