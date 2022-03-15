use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;

pub async fn random_string(size: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect()
}
