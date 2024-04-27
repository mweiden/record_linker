use rand::{distributions::Uniform, Rng};

pub fn random_string(len: usize) -> String {
    // Define the character set to sample from: 0-9a-z
    let chars: Vec<char> = ('0'..'9').chain('a'..'z').collect();

    // Create a uniform distribution to sample indices of the character set
    let dist = Uniform::new(0, chars.len());

    // Create a random generator (thread-local)
    let mut rng = rand::thread_rng();

    // Generate the random string of the specified length
    (0..len)
        .map(|_| chars[rng.sample(dist)]) // Sample a character for each position
        .collect()
}
