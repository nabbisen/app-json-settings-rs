#[cfg(test)]
mod tests {
    use crate::exec_dir_filepath;

    #[test]
    fn print_exec_dir_filepath() {
        let filepath = exec_dir_filepath();
        
        // cargo test -- --nocapture
        println!("{:?}", filepath);

        assert!(filepath.as_os_str().to_str().is_some())
    }
}
