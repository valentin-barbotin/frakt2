use rand::Rng;

pub fn random_string(max: u16) -> String {
    let mut rng = rand::thread_rng();
    let name: String = (0..max)
        .map(|_| rng.gen_range(b'a'..=b'z') as char)
        .collect();
    name
}
