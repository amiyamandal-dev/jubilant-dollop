use rand::Rng;

pub fn generate_id() -> Vec<u8> {
    let random_bytes = rand::thread_rng().gen::<[u8; 20]>();
    random_bytes.to_vec()
}
