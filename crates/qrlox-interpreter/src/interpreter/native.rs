// #[native]
pub fn clock() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time cannot be before unix beginning")
        .as_secs_f64()
}
