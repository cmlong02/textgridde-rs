use std::{
    collections::VecDeque,
    fs::File,
    io::{BufRead, BufReader, Error, ErrorKind, Read, Result},
    path::PathBuf,
};

use regex::Regex;

use crate::input::Source;

/// Pull the next number from the `VecDeque` of `String`s.
///
/// # Arguments
///
/// * `textgrid_data` - A mutable reference to a `VecDeque` of `String`s.
///
/// # Returns
///
/// The next number in the `VecDeque` as the specified type.
pub fn pull_next_number<T>(textgrid_data: &mut VecDeque<String>) -> Result<T>
where
    T: std::str::FromStr,
{
    let re = Regex::new(r"\d+(\.\d+)?").unwrap(); // Unwrap is safe here

    while let Some(line) = textgrid_data.pop_front() {
        if let Some(captures) = re.captures(&line) {
            if let Some(matched) = captures.get(0) {
                return matched.as_str().to_string().parse::<T>().map_err(|_| {
                    Error::new(
                        ErrorKind::InvalidData,
                        format!(
                            "TextGrid malformed; Unable to parse expected number \"{}\" as {}",
                            matched.as_str(),
                            std::any::type_name::<T>()
                        ),
                    )
                });
            }
        }
    }

    Err(Error::new(
        ErrorKind::InvalidData,
        format!(
            "TextGrid malformed; Unable to find expected {}",
            std::any::type_name::<T>()
        ),
    ))
}

/// Process lines of text, removing quotes and non-numeric characters.
///
/// # Arguments
///
/// * `lines` - A vector of strings to process.
///
/// # Returns
///
/// A vector of strings with quotes removed and non-numeric characters removed.
pub fn process_lines(lines: &[String]) -> Vec<String> {
    let split_lines: Vec<String> = lines
        .iter()
        .flat_map(|line| split_line_with_regex(line).into_iter())
        .collect();

    let mut processed_lines: Vec<String> = Vec::new();

    for line in &split_lines {
        if line.starts_with('"') && line.ends_with('"') && line.len() > 1 {
            processed_lines.push(line[1..line.len() - 1].to_string());
        } else if line
            .chars()
            .all(|character| character.is_numeric() || character == '.')
        {
            processed_lines.push(line.to_string());
        }
    }

    processed_lines
}

/// Split a line by spaces, but keep quoted strings together.
///
/// # Arguments
///
/// * `line` - A line of text to split
///
/// # Returns
///
/// A vector of strings split by spaces, but keeping quoted strings together.
fn split_line_with_regex(line: &str) -> Vec<String> {
    // Combined regex to split spaces not within quotes
    let re = Regex::new(r#""[^"]*"|\S+"#).unwrap();
    let split = re
        .captures_iter(line)
        .flat_map(|captures| {
            captures
                .iter()
                .filter_map(|capture| capture.map(|m| m.as_str().to_string()))
                .collect::<Vec<String>>()
        })
        .collect::<Vec<String>>();

    split
}

/// Gets the content of a file or stream.
///
/// # Arguments
///
/// * `source` - One of the following:
///
/// # Returns
///
/// A `Result` containing a tuple of a vector of strings and a string if successful, or an `std::io::Error` if parsing failed.
pub fn get_file_content(source: Source) -> Result<(Vec<String>, String)> {
    match source {
        Source::Path(path) => {
            let mut file = File::open(path.clone())?;

            let mut content_joined = String::default();
            file.read_to_string(&mut content_joined)?;
            let content = content_joined
                .split('\n')
                .map(std::string::ToString::to_string)
                .collect::<Vec<String>>();

            let name = path
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
                .into();

            Ok((content, name))
        }
        Source::String(string) => {
            if PathBuf::from(&string).is_file() {
                return get_file_content(Source::Path(string.into()));
            }

            let content = string
                .split('\n')
                .map(std::string::ToString::to_string)
                .collect::<Vec<String>>();

            let name = "New TextGrid".to_string();

            Ok((content, name))
        }
        Source::StringVector(string_vector) => Ok((string_vector, "New TextGrid".to_string())),
        Source::Stream(stream) => {
            // Wrap the stream in a BufReader to use the lines method
            let reader = BufReader::new(stream);
            let parsed_content: Result<Vec<String>> = reader.lines().collect();
            let content = parsed_content?;
            let name = "New TextGrid".to_string();

            Ok((content, name))
        }
        Source::File(file) => {
            // Wrap the file in a BufReader to use the lines method
            let reader = BufReader::new(file);
            let parsed_content: Result<Vec<String>> = reader.lines().collect();
            let content = parsed_content?;
            let name = "New TextGrid".to_string();

            Ok((content, name))
        }
    }
}

#[cfg(test)]
mod test_utilities {
    use crate::{input::Source, utilities};
    use std::{collections::VecDeque, io::Cursor};

    #[test]
    fn pull_next_number() {
        let mut textgrid_data = VecDeque::new();
        textgrid_data.push_back("xmin = 0".to_string());

        let expected = 0;
        assert_eq!(
            utilities::pull_next_number::<i32>(&mut textgrid_data).unwrap(),
            expected
        );
    }

    #[test]
    fn split_line_with_regex() {
        let line = "one two \"three four\" five";
        let expected = vec!["one", "two", "\"three four\"", "five"];
        assert_eq!(utilities::split_line_with_regex(line), expected);
    }

    #[test]
    fn process_lines() {
        let lines = vec![
            "one two \"three four\" five".to_string(),
            "1 2 3.4 5".to_string(),
        ];
        let expected = vec!["three four", "1", "2", "3.4", "5"];
        assert_eq!(utilities::process_lines(&lines), expected);
    }

    #[test]
    fn get_file_content() {
        let content = "xmin = 0\nxmax = 10";
        let source = Source::Stream(Box::new(Cursor::new(content)));
        let (content, name) = utilities::get_file_content(source).unwrap();
        let expected_content = vec!["xmin = 0".to_string(), "xmax = 10".to_string()];
        let expected_name = "New TextGrid".to_string();
        assert_eq!(content, expected_content);
        assert_eq!(name, expected_name);
    }
}
