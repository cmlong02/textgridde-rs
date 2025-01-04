use std::{
    collections::VecDeque,
    fs::File,
    io::{BufReader, Error, ErrorKind, Read, Result},
    path::PathBuf,
};

use encoding_rs::Encoding;
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
///     * `Source::StringVector` - A vector of strings (what every other source is converted to).
///     * `Source::Stream` - A stream of text (what path/file is converted to).
///     * `Source::String` - A string that may be a path to a file or the content itself.
///     * `Source::Path` - A path to a file.
///     * `Source::File` - A file.
/// * `name` - The file name; only passed during recursion.
///
/// # Returns
///
/// A `Result` containing a tuple of a vector of strings and a string if successful, or an `std::io::Error` if parsing failed.
pub fn get_file_content(source: Source, name: Option<String>) -> Result<(Vec<String>, String)> {
    match source {
        Source::StringVector(string_vector) => {
            Ok((string_vector, name.unwrap_or("New TextGrid".to_string())))
        }
        Source::String(string) => {
            if PathBuf::from(&string).is_file() {
                return get_file_content(Source::Path(string.into()), None);
            }
            let content = string
                .split('\n')
                .map(std::string::ToString::to_string)
                .collect::<Vec<String>>();
            let name = "New TextGrid".to_string();

            get_file_content(Source::StringVector(content), Some(name))
        }
        Source::Stream(mut stream) => {
            // Use encoding_rs to detect the BOM and convert to UTF-8 if necessary, since TextGrid files can sometimes be encoded in UTF-16.

            let mut buffer = Vec::new();
            stream.read_to_end(&mut buffer)?;

            let (encoding, _) = Encoding::for_bom(&buffer).unwrap_or((encoding_rs::UTF_8, 0));
            let content = encoding.decode(&buffer).0;

            let content = content
                .split('\n')
                .map(std::string::ToString::to_string)
                .collect::<Vec<String>>();

            get_file_content(
                Source::StringVector(content),
                Some(name.unwrap_or("New TextGrid".to_string())),
            )
        }
        Source::Path(path) => {
            let file = File::open(path.clone())?;
            let name = path
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("New Textgrid"))
                .to_str()
                .unwrap()
                .to_string();

            get_file_content(Source::File(file), Some(name))
        }
        Source::File(file) => {
            // Wrap the file in a BufReader to recurse with Source::Stream
            let reader = BufReader::new(file);
            let stream = Box::new(reader);

            get_file_content(
                Source::Stream(stream),
                Some(name.unwrap_or_else(|| "New TextGrid".to_string())),
            )
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
        let (content, name) = utilities::get_file_content(source, None).unwrap();
        let expected_content = vec!["xmin = 0".to_string(), "xmax = 10".to_string()];
        let expected_name = "New TextGrid".to_string();
        assert_eq!(content, expected_content);
        assert_eq!(name, expected_name);
    }

    #[test]
    fn utf16() {
        let content = utilities::get_file_content(Source::Path("./example/utf16.TextGrid".into()), None).unwrap().0;
        let expected_content = utilities::get_file_content(Source::Path("./example/long.TextGrid".into()), None).unwrap().0;

        assert_eq!(content, expected_content);
    }
}
