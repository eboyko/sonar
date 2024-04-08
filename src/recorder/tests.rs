#[cfg(test)]
mod tests {
    const TEMPORARY_STORAGE_PATH: &str = "tmp/storage";

    #[test]
    fn build() {
        crate::recorder::build(TEMPORARY_STORAGE_PATH.to_string()).unwrap();
        assert!(std::path::Path::new(TEMPORARY_STORAGE_PATH).exists());
        std::fs::remove_dir_all(TEMPORARY_STORAGE_PATH).unwrap();
    }
}
