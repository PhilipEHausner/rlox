use log::error;
use std::cmp::min;

pub struct LineInformation {
    offset: usize,
    length: usize,
}

impl LineInformation {
    pub fn new(offset: usize, length: usize) -> LineInformation {
        LineInformation { offset, length }
    }
}

pub struct ErrorHandler {
    pub had_error: bool,
    code: String,
}

impl ErrorHandler {
    pub fn init_logging() -> Result<(), fern::InitError> {
        fern::Dispatch::new()
            .format(|out, msg, record| out.finish(format_args!("{}: {}", record.level(), msg)))
            .level(log::LevelFilter::Debug)
            .chain(std::io::stdout())
            .apply()?;
        Ok(())
    }

    pub fn new(code: &String) -> ErrorHandler {
        ErrorHandler {
            had_error: false,
            code: code.clone(),
        }
    }

    pub fn report_error(&self, error_msg: &str, line_information: &LineInformation) {
        assert!(self.code.len() >= line_information.offset + line_information.length);

        let msg = self.get_error_message(error_msg, line_information);
        error!("{}", msg);
    }

    fn get_error_message(&self, error_msg: &str, line_information: &LineInformation) -> String {
        let mut result = format!("{error_msg}\n").to_string();

        let line = &self.code[..=line_information.offset]
            .chars()
            .filter(|it| it == &'\n')
            .count()
            + 1;
        let indentation = (line.checked_ilog10().unwrap_or(0) + 3) as usize;
        let (code_line, column_offset, column_end) =
            self.get_line_content_and_column_offset(line_information.offset);

        result += &format!("{}|\n", " ".repeat(indentation));
        result += &format!(" {} | {}\n", line, code_line);

        let marker_start = " ".repeat(column_offset);

        // Only mark until end of line if error goes over multiple lines.
        let max_marker_length = min(
            line_information.length,
            column_end - line_information.offset,
        );
        let marker = "^".repeat(max_marker_length);
        result += &format!("{}| {}{}\n", " ".repeat(indentation), marker_start, marker);

        // If error goes over multiple lines, we report this to the user.
        if max_marker_length < line_information.length {
            result += &format!(
                "{}| --> Error continues in next line.\n",
                " ".repeat(indentation)
            );
        }
        result
    }

    fn get_line_content_and_column_offset(&self, offset: usize) -> (String, usize, usize) {
        // Left boundary of the code line.
        let mut left = 0;
        for (idx, c) in self.code.chars().enumerate() {
            if idx == offset {
                break;
            }
            if c == '\n' {
                left = idx + 1;
            }
        }

        // The offset where in the line the marked error is located.
        let column_offset = offset - left;

        // Right boundary of the code line.
        let mut right = self.code.len();
        for (idx, c) in self.code.chars().skip(offset).enumerate() {
            if c == '\n' {
                right = offset + idx;
                break;
            }
        }

        (self.code[left..right].to_string(), column_offset, right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> String {
        "fn my_function() -> usize {\n    10 + 10\n}  // A function".to_string()
    }

    #[test]
    fn test_get_error_message_first_token() {
        let input = input();
        let error_handler = ErrorHandler::new(&input);
        let li = LineInformation::new(0, 2);

        let msg = error_handler.get_error_message("An error occurred.", &li);

        assert_eq!(
            msg,
            "An error occurred.\n   |\n 1 | fn my_function() -> usize {\n   | ^^\n"
        )
    }

    #[test]
    fn test_get_error_message_first_line() {
        let input = input();
        let error_handler = ErrorHandler::new(&input);
        let li = LineInformation::new(3, 13);

        let msg = error_handler.get_error_message("An error occurred.", &li);

        assert_eq!(
            msg,
            "An error occurred.\n   |\n 1 | fn my_function() -> usize {\n   |    ^^^^^^^^^^^^^\n"
        )
    }

    #[test]
    fn test_get_error_message_first_token_second_line() {
        let input = input();
        let error_handler = ErrorHandler::new(&input);
        let li = LineInformation::new(28, 4);

        let msg = error_handler.get_error_message("An error occurred.", &li);

        assert_eq!(
            msg,
            "An error occurred.\n   |\n 2 |     10 + 10\n   | ^^^^\n"
        )
    }

    #[test]
    fn test_get_error_message_second_line() {
        let input = input();
        let error_handler = ErrorHandler::new(&input);
        let li = LineInformation::new(35, 1);

        let msg = error_handler.get_error_message("An error occurred.", &li);

        assert_eq!(
            msg,
            "An error occurred.\n   |\n 2 |     10 + 10\n   |        ^\n"
        )
    }

    #[test]
    fn test_get_error_message_last_line() {
        let input = input();
        let error_handler = ErrorHandler::new(&input);
        let li = LineInformation::new(43, 2);

        let msg = error_handler.get_error_message("An error occurred.", &li);

        assert_eq!(
            msg,
            "An error occurred.\n   |\n 3 | }  // A function\n   |    ^^\n"
        )
    }

    #[test]
    fn test_get_error_message_last_token_last_line() {
        let input = input();
        let error_handler = ErrorHandler::new(&input);
        let li = LineInformation::new(48, 8);

        let msg = error_handler.get_error_message("An error occurred.", &li);

        assert_eq!(
            msg,
            "An error occurred.\n   |\n 3 | }  // A function\n   |         ^^^^^^^^\n"
        )
    }

    #[test]
    fn test_multiple_lines_error() {
        let input = input();
        let error_handler = ErrorHandler::new(&input);
        let li = LineInformation::new(37, 3);

        let msg = error_handler.get_error_message("An error occurred.", &li);

        assert_eq!(
            msg,
            "An error occurred.\n   |\n 2 |     10 + 10\n   |          ^^\n   | --> Error continues in next line.\n"
        )
    }
}
