use rand::{distributions::Alphanumeric, Rng};

pub fn random_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .map(|c| char::from(c).to_ascii_lowercase())
        .take(len)
        .collect()
}
