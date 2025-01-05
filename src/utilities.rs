use std::{
    collections::VecDeque,
    io::{Error, ErrorKind, Result},
    sync::LazyLock,
};

use regex::Regex;

static NEXT_NUMBER_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\d+(\.\d+)?").unwrap());
static QUOTED_STRING_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#""[^"]*"|\S+"#).unwrap());

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
    while let Some(line) = textgrid_data.pop_front() {
        if let Some(captures) = NEXT_NUMBER_RE.captures(&line) {
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
    let split = QUOTED_STRING_RE
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

#[cfg(test)]
mod test_utilities {
    use crate::utilities;
    use std::collections::VecDeque;

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
}
