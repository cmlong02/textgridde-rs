#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]

use std::{
    collections::VecDeque,
    fs::File,
    io::{BufRead, BufReader, Error, ErrorKind, Read, Result},
    path::PathBuf,
};

mod input;
pub mod textgrid;
mod utilities;

use input::Source;
use textgrid::{Interval, IntervalTier, Point, PointTier, TextGrid, Tier};

/// Parses a Praat `.TextGrid` file into a `textgridde::Textgrid` struct.
///
/// # Arguments
///
/// * `input` - One of the following:
///                 * A path to a `.TextGrid` file.
///                 * A string containing the entire `TextGrid` file.
///                 * A vector of strings containing the lines of a `.TextGrid` file.
///                 * A stream containing the contents of a `.TextGrid` file.
/// * `print_warnings?` - An optional boolean indicating whether to print warnings.
///
/// # Returns
///
/// A `Result` containing a `textgridde::TextGrid` struct if successful, or a `std::io::Error` if parsing failed.
///
/// # Errors
///
/// If a `TextGrid` is malformed irrecoverably, an `std::io::Error` is returned. This can be for one of the following reasons:
///     * The file does not start with the correct `File type` and `Object class` (`"ooTextFile"` and `"TextGrid"` respectively).
///     * The `xmin` and `xmax` values are not present or cannot be parsed as floats.
///     * The `exists` value is not present or is not equal to "exists".
///     * A tier type is not recognized.
pub fn parse_textgrid<T>(input: T, print_warnings: Option<bool>) -> Result<TextGrid>
where
    T: Into<Source>,
{
    let input_source: Source = input.into();
    let mut content;
    match input_source {
        Source::Path(path) => {
            let mut file = File::open(path)?;
            let mut content_joined = String::default();
            file.read_to_string(&mut content_joined)?;
            content = content_joined
                .split('\n')
                .map(std::string::ToString::to_string)
                .collect::<Vec<String>>();
        }
        Source::String(string) => {
            if let Ok(path) = PathBuf::from(&string).canonicalize() {
                if path.is_file() {
                    return parse_textgrid(path, print_warnings);
                }
            }

            content = string
                .split('\n')
                .map(std::string::ToString::to_string)
                .collect::<Vec<String>>();
        }
        Source::StringVector(string_vector) => {
            // Jump to string parsing
            content = string_vector;
        }
        Source::Stream(stream) => {
            // Wrap the stream in a BufReader to use the lines method
            let reader = BufReader::new(stream);
            let parsed_content: Result<Vec<String>> = reader.lines().collect();
            content = parsed_content?;
        }
    }

    // Clean up the content by removing empty or whitespace-only lines
    content.retain(|s| !s.trim().is_empty());

    // Split lines with spaces not inside quotation marks into their own elements
    content = utilities::split_lines_with_spaces(&content);

    // Convert into a VecDeque for efficient popping from the front
    let mut textgrid_data: VecDeque<_> = content.into();

    // Verify the start of the TextGrid file, ensuring "File type" and "Object class" exist
    let textgrid_data = verify_start_of_textgrid(&mut textgrid_data)?;

    let tg_xmin = textgrid_data
        .pop_front()
        .ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                "TextGrid malformed; early EOF expecting `xmin`",
            )
        })?
        .chars()
        .filter(|c| c.is_numeric() || *c == '.')
        .collect::<String>()
        .parse::<f64>()
        .map_err(|_| {
            Error::new(
                ErrorKind::InvalidData,
                "TextGrid malformed; could not parse `xmin` as a float",
            )
        })?;

    let tg_xmax = textgrid_data
        .pop_front()
        .ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                "TextGrid malformed; early EOF expecting `xmax`",
            )
        })?
        .chars()
        .filter(|c| c.is_numeric() || *c == '.')
        .collect::<String>()
        .parse::<f64>()
        .map_err(|_| {
            Error::new(
                ErrorKind::InvalidData,
                "TextGrid malformed; could not parse `xmax` as a float",
            )
        })?;

    let exists =
        utilities::pull_string_between_char(textgrid_data, '<', Some('>')).map_err(|_| {
            Error::new(
                ErrorKind::InvalidData,
                "TextGrid malformed; `exists` not found",
            )
        })?;
    if exists != "exists" {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "TextGrid malformed; `exists` incorrect",
        ));
    }

    let parsed_textgrid = parse_tiers(textgrid_data, tg_xmin, tg_xmax, print_warnings)?;

    Ok(TextGrid::new(tg_xmin, tg_xmax, parsed_textgrid))
}

fn verify_start_of_textgrid(textgrid_data: &mut VecDeque<String>) -> Result<&mut VecDeque<String>> {
    if utilities::pull_string_between_char(textgrid_data, '"', None)? != "ooTextFile" {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "TextGrid malformed; `File type` incorrect",
        ));
    }

    if utilities::pull_string_between_char(textgrid_data, '"', None)? != "TextGrid" {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "TextGrid malformed; `Object class` incorrect",
        ));
    }

    Ok(textgrid_data)
}

fn parse_tiers(
    data: &mut VecDeque<String>,
    tg_xmin: f64,
    tg_xmax: f64,
    print_warnings: Option<bool>,
) -> Result<Vec<Tier>> {
    let mut tiers = Vec::<Tier>::new();

    while !data.is_empty() {
        let tier_type = utilities::pull_string_between_char(data, '"', None)?;
        let tier_name = utilities::pull_string_between_char(data, '"', None)?;
        let size = utilities::pull_next_number::<usize>(data)?;
        let mut counter = size;

        match tier_type.as_str() {
            "IntervalTier" => {
                let mut new_tier: IntervalTier =
                    IntervalTier::new(tier_name.clone(), tg_xmin, tg_xmax, Vec::<Interval>::new());

                while data.front().is_some()
                    && !["\"IntervalTier\"".to_string(), "\"TextTier\"".to_string()]
                        .contains(data.front().unwrap())
                {
                    new_tier.push_interval(parse_interval(data)?, print_warnings);
                    counter -= 1;
                }
                if counter != 0 && print_warnings.is_some_and(|b| b) {
                    eprintln!(
                        "Warning: Tier `{}` has a size of {} but {} intervals were found",
                        tier_name,
                        size,
                        size - counter
                    );
                }
                tiers.push(Tier::IntervalTier(new_tier));
            }
            "TextTier" => {
                let mut new_tier =
                    PointTier::new(tier_name.clone(), tg_xmin, tg_xmax, Vec::<Point>::new());

                while data.front().is_some()
                    && !["\"IntervalTier\"".to_string(), "\"TextTier\"".to_string()]
                        .contains(data.front().unwrap())
                {
                    new_tier.push_point(parse_point(data)?, print_warnings);
                    counter -= 1;
                }
                if counter != 0 && print_warnings.is_some_and(|b| b) {
                    eprintln!(
                        "Warning: Tier `{}` has a size of {} but {} points were found",
                        tier_name,
                        size,
                        size - counter
                    );
                }
                tiers.push(Tier::PointTier(new_tier));
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "TextGrid malformed; Invalid tier type",
                ));
            }
        }
    }

    Ok(tiers)
}

fn parse_interval(data: &mut VecDeque<String>) -> Result<Interval> {
    let xmin = utilities::pull_next_number::<f64>(data)?;
    let xmax = utilities::pull_next_number::<f64>(data)?;
    let text = utilities::pull_string_between_char(data, '"', None)?;

    Ok(Interval::new(xmin, xmax, text))
}

fn parse_point(data: &mut VecDeque<String>) -> Result<Point> {
    let number = utilities::pull_next_number::<f64>(data)?;
    let mark = utilities::pull_string_between_char(data, '"', None)?;

    Ok(Point::new(number, mark))
}
