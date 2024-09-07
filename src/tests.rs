#[cfg(test)]
mod tests {
    use crate::default_filepath;

    #[test]
    fn print_default_filepath() {
        let filepath = default_filepath();
        println!("{:?}", filepath);
    }
}
