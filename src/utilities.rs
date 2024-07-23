use std::{
    collections::VecDeque,
    io::{Error, ErrorKind, Result},
};

use regex::Regex;

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

pub fn process_lines(lines: &[String]) -> Vec<String> {
    let split_lines: Vec<String> = lines
        .iter()
        .flat_map(|line| split_line_with_regex(line).into_iter())
        .collect();

    let mut processed_lines: Vec<String> = Vec::new();

    for line in &split_lines {
        if line.starts_with('"') && line.ends_with('"') {
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
