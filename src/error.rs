#[derive(Debug)]
pub struct ErrorHandler {
    source: String,
}

impl ErrorHandler {
    pub fn new(source: String) -> Self {
        ErrorHandler { source }
    }

    pub fn error(&self, index_in_source: usize, message: &str) {
        let (line_no, line_start_index, line) = self.get_line(index_in_source);
        eprintln!("Error [line {}]: {}", line_no, message);
        eprintln!("{}", line);
        for _ in 1..(index_in_source - line_start_index) {
            eprint!(" ")
        }
        eprintln!("^");
    }

    fn get_line(&self, index_in_source: usize) -> (usize, usize, &str) {
        let mut index_counter = 0;
        let mut line_counter = 1;
        for line in self.source.split("\n") {
            if index_counter + line.len() >= index_in_source {
                return (line_counter, index_counter, line);
            }
            index_counter += line.len();
            line_counter += 1;
        }
        panic!("Line not found. Index: {}", index_in_source);
    }
}
