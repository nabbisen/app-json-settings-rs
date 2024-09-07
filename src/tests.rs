#[cfg(test)]
mod tests {
    use crate::{config_dir_filepath, exe_dir_filepath};

    #[test]
    fn print_exec_dir_filepath() {
        let filepath = exe_dir_filepath("test.json");

        // cargo test -- --nocapture
        println!("{:?}", filepath);

        assert!(filepath.as_os_str().to_str().is_some())
    }

    #[test]
    fn print_config_dir_filepath() {
        let filepath = config_dir_filepath("test2.json");

        // cargo test -- --nocapture
        println!("{:?}", filepath);

        assert!(filepath.as_os_str().to_str().is_some())
    }
}
