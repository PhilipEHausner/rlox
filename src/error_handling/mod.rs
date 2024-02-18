pub struct ErrorLineInformation {
    line: usize,
    column_start: usize,
    length: usize,
}

impl ErrorLineInformation {
    pub fn new(line: usize, column_start: usize, length: usize) -> ErrorLineInformation {
        ErrorLineInformation{ line , column_start, length}
    }
}

pub struct ErrorHandler {
    pub had_error: bool,
    code: Vec<String>,
}

impl ErrorHandler {
    pub fn new(code: &String) -> ErrorHandler {
        let src_code: Vec<String> = code.split("\n").map(|it| {it.to_string()}).collect();
        ErrorHandler { had_error: false, code: src_code }
    }
    pub fn report_error(&self, error_msg: &str, line_information: &ErrorLineInformation) {
        println!("Error: {error_msg}");
        assert!(self.code.len() >= line_information.line);

        let indentation = (line_information.line.checked_ilog10().unwrap_or(0) + 3) as usize;
        let code_line = &self.code[line_information.line - 1];
        println!("{}|", " ".repeat(indentation));
        println!(" {} | {}", line_information.line, code_line);

        let marker_start = " ".repeat(line_information.column_start);
        let marker = "^".repeat(line_information.length);
        println!("{}|{}{}", " ".repeat(indentation), marker_start, marker);
        println!();
    }
}
