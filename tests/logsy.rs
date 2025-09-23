#[cfg(test)]
mod tests {
    use log::{debug, info, LevelFilter};
    use tempdir::TempDir;
    #[test]
    fn test_logsy() {
        let dir = TempDir::new("my_directory_prefix").unwrap();
        let filename = dir.path().join("foo.log");
        logsy::set_filename(Some(filename.to_str().unwrap()));
        info!("Test123");
        let contents = std::fs::read_to_string(&filename).unwrap();
        assert!(contents.contains("Test123"));

        info!("Test125");
        debug!("Test126");
        let contents = std::fs::read_to_string(&filename).unwrap();
        assert!(contents.contains("Test123"));
        assert!(contents.contains("Test125"));
        assert!(contents.contains("INFO"));
        assert!(!contents.contains("Test126"));
        assert!(!contents.contains("DEBUG"));

        logsy::set_level(LevelFilter::Debug);
        debug!("Test126");
        let contents = std::fs::read_to_string(&filename).unwrap();
        assert!(contents.contains("Test123"));
        assert!(contents.contains("Test125"));
        assert!(contents.contains("Test126"));
        assert!(contents.contains("INFO"));
        assert!(contents.contains("DEBUG"));

        let filename = dir.path().join("foo2.log");
        logsy::set_filename(Some(filename.to_str().unwrap()));
        info!("Test127");
        info!("Test128");
        info!("Test129");
        let contents = std::fs::read_to_string(&filename).unwrap();
        assert!(contents.contains("Test127"));
        assert!(contents.contains("Test128"));
        assert!(contents.contains("Test129"));
    }
}
