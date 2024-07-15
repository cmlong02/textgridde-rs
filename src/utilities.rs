use std::{
    collections::VecDeque,
    io::{Error, ErrorKind, Result},
};

use regex::Regex;

pub fn pull_string_between_char(
    textgrid_data: &mut VecDeque<String>,
    delimiter1: char,
    delimiter2: Option<char>,
) -> Result<String> {
    let delimiter_combined =
        delimiter2.map_or_else(|| delimiter1.to_string(), |d| format!("{delimiter1}{d}"));

    let delimiter2_unwrapped = delimiter2.unwrap_or(delimiter1);

    let re = Regex::new(&format!(
        r#"{delimiter1}([^{delimiter_combined}]+){delimiter2_unwrapped}"#
    ))
    .unwrap(); // Unwrap is safe here

    while let Some(line) = textgrid_data.pop_front() {
        if let Some(captures) = re.captures(&line) {
            if let Some(matched) = captures.get(1) {
                return Ok(matched.as_str().to_string());
            }
        }
    }

    Err(Error::new(
        ErrorKind::InvalidData,
        format!("TextGrid malformed; could not find matching string between `{delimiter1}` and `{delimiter2_unwrapped}`"),
    ))
}

pub fn pull_next_number<T>(textgrid_data: &mut VecDeque<String>) -> Result<T>
where
    T: std::str::FromStr,
{
    let re = Regex::new(r"\b\d+(\.\d+)?\b").unwrap(); // Unwrap is safe here

    while let Some(line) = textgrid_data.pop_front() {
        if let Some(captures) = re.captures(&line) {
            if let Some(matched) = captures.get(0) {
                return matched.as_str().to_string().parse::<T>().map_err(|_| {
                    Error::new(
                        ErrorKind::InvalidData,
                        format!(
                            "TextGrid malformed; Unable to parse number as {}",
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

pub fn split_lines_with_spaces(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        .flat_map(|line| split_line_with_regex(line))
        .collect()
}

fn split_line_with_regex(line: &str) -> Vec<String> {
    // Combined regex to split spaces not within quotes
    let re = Regex::new(r#""[^"]*"|\S+"#).unwrap();
    let split = re
        .find_iter(line)
        .map(|mat| mat.as_str().to_string())
        .collect::<Vec<String>>();

    if split.iter().any(|s| s.starts_with('!')) {
        return split
            .iter()
            .flat_map(|s| s.split('!').map(std::string::ToString::to_string))
            .take(1)
            .collect();
    }

    split
}
