#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(clippy::cargo)]

use std::{
    collections::VecDeque,
    io::{Error, ErrorKind, Result},
};

mod input;
pub mod interval;
pub mod point;
pub mod textgrid;
mod utilities;

use input::Source;
use interval::{Interval, Tier as IntervalTier};
use point::{Point, Tier as PointTier};
use textgrid::{TextGrid, Tier};
use utilities::get_file_content;

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
pub fn parse_textgrid<I, W>(input: I, print_warnings: W) -> Result<TextGrid>
where
    I: Into<Source>,
    W: Into<Option<bool>> + Copy,
{
    let input_source: Source = input.into();

    let (mut content, name) = get_file_content(input_source)?;

    // Clean up the content by removing empty or whitespace-only lines
    content.retain(|s| !s.trim().is_empty());

    // Iterate over lines, removing comments (a "!" after an odd number of quotation marks and everything after it)
    for line in &mut content {
        let mut quote_count = 0;
        let mut quote_indices = Vec::<usize>::new();
        for (i, c) in line.chars().enumerate() {
            if c == '"' {
                quote_count += 1;
                quote_indices.push(i);
            }
            if c == '!' && quote_count % 2 != 0 {
                *line = line[..quote_indices[quote_indices.len() - 2]].to_string();
                break;
            }
        }
    }

    // Split lines with spaces not inside quotation marks into their own elements
    content = utilities::process_lines(&content);

    // Convert into a VecDeque for efficient popping from the front
    let mut textgrid_data: VecDeque<String> = VecDeque::from(content);

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

    let parsed_textgrid = parse_tiers(textgrid_data, tg_xmin, tg_xmax, print_warnings)?;

    Ok(TextGrid::new(tg_xmin, tg_xmax, parsed_textgrid, name))
}

fn verify_start_of_textgrid(textgrid_data: &mut VecDeque<String>) -> Result<&mut VecDeque<String>> {
    let file_type = textgrid_data.pop_front().unwrap_or_default();
    if file_type != "ooTextFile" {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "TextGrid malformed; `File type` incorrect: expected `ooTextFile`, got {file_type}"
            ),
        ));
    }

    let object_class = textgrid_data.pop_front().unwrap_or_default();
    if object_class != "TextGrid" {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("TextGrid malformed; `Object class` incorrect: expected `TextGrid`, got {object_class}"),
        ));
    }

    Ok(textgrid_data)
}

/// Parses tiers from a `TextGrid` file.
///
/// # Arguments
///
/// * `data` - A mutable reference to a `VecDeque<String>` containing the lines of a `TextGrid` file.
/// * `tg_xmin` - The `xmin` value of the `TextGrid`.
/// * `tg_xmax` - The `xmax` value of the `TextGrid`.
/// * `warn` - An optional boolean indicating whether to print warnings.
///
/// # Returns
///
/// A `Result` containing a vector of `textgridde::Tier` structs if successful, or an `std::io::Error` if parsing failed.
fn parse_tiers<W: Into<Option<bool>> + Copy>(
    data: &mut VecDeque<String>,
    tg_xmin: f64,
    tg_xmax: f64,
    warn: W,
) -> Result<Vec<Tier>> {
    let mut tiers = Vec::<Tier>::new();

    let num_tiers = utilities::pull_next_number::<i64>(data)?;
    let mut num_tier_counter = 0;

    while !data.is_empty() {
        num_tier_counter += 1;

        let tier_type = data.pop_front().ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                "TextGrid malformed; early EOF expecting tier type",
            )
        })?;
        let tier_name = data.pop_front().ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                "TextGrid malformed; early EOF expecting tier name",
            )
        })?;

        let xmin = utilities::pull_next_number::<f64>(data)?;
        let xmax = utilities::pull_next_number::<f64>(data)?;

        if warn.into().unwrap_or_default() {
            if xmin < tg_xmin {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "TextGrid malformed; tier {tier_name} `xmin` less than TextGrid `xmin`",
                ));
            }
            if xmax > tg_xmax {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "TextGrid malformed; tier {tier_name} `xmax` greater than TextGrid `xmax`",
                ));
            }
        }

        let tier_size = utilities::pull_next_number::<i64>(data)?;
        let mut tier_size_counter = 0;

        match tier_type.as_str() {
            "IntervalTier" => {
                let mut new_tier: IntervalTier =
                    IntervalTier::new(tier_name.clone(), xmin, xmax, Vec::<Interval>::new());

                while data.front().is_some()
                    && !["IntervalTier".to_string(), "TextTier".to_string()]
                        .contains(data.front().unwrap())
                {
                    new_tier.push_interval(parse_interval(data)?, warn);
                    tier_size_counter += 1;
                }
                if warn.into().unwrap_or_default() && tier_size != tier_size_counter {
                    eprintln!(
                        "Warning: Tier `{tier_name}` has a size of {tier_size} but {tier_size_counter} intervals were found",
                    );
                }
                tiers.push(Tier::IntervalTier(new_tier));
            }
            "TextTier" => {
                let mut new_tier =
                    PointTier::new(tier_name.clone(), xmin, xmax, Vec::<Point>::new());

                while data.front().is_some()
                    && !["\"IntervalTier\"".to_string(), "\"TextTier\"".to_string()]
                        .contains(data.front().unwrap())
                {
                    new_tier.push_point(parse_point(data)?, warn);
                    tier_size_counter += 1;
                }
                if warn.into().unwrap_or_default() && tier_size != tier_size_counter {
                    eprintln!(
                        "Warning: Tier `{tier_name}` has a size of {tier_size} but {tier_size_counter} points were found",
                    );
                }
                tiers.push(Tier::PointTier(new_tier));
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("TextGrid malformed; Invalid tier type: {tier_type}"),
                ));
            }
        }
    }

    if num_tiers != num_tier_counter && warn.into().unwrap_or_default() {
        eprintln!(
            "Warning: TextGrid has a size of {num_tiers} but {num_tier_counter} tiers were found",
        );
    }

    Ok(tiers)
}

/// Parses an `Interval` from a `TextGrid` file.
///
/// # Arguments
///
/// * `data` - A mutable reference to a `VecDeque<String>` containing the lines of a `TextGrid` file.
///
/// # Returns
///
/// A `Result` containing an `Interval` struct if successful, or an `std::io::Error` if parsing failed.
fn parse_interval(data: &mut VecDeque<String>) -> Result<Interval> {
    let xmin = utilities::pull_next_number::<f64>(data)?;
    let xmax = utilities::pull_next_number::<f64>(data)?;
    let text = data.pop_front().unwrap_or_default();

    Ok(Interval::new(xmin, xmax, text))
}

/// Parses a `Point` from a `TextGrid` file.
///
/// # Arguments
///
/// * `data` - A mutable reference to a `VecDeque<String>` containing the lines of a `TextGrid` file.
///
/// # Returns
///
/// A `Result` containing a `Point` struct if successful, or an `std::io::Error` if parsing failed.
fn parse_point(data: &mut VecDeque<String>) -> Result<Point> {
    let number = utilities::pull_next_number::<f64>(data)?;
    let mark = data.pop_front().unwrap_or_default();

    Ok(Point::new(number, mark))
}

#[cfg(test)]
mod test {
    use std::collections::VecDeque;

    use crate::input::Source;

    use super::parse_textgrid;

    const TEXTGRID: &str = "File type = \"ooTextFile\"\nObject class = \"TextGrid\"\n\nxmin = 0\nxmax = 2.3\ntiers? <exists>\nsize = 3\nitem []:\n\titem [1]:\n\t\tclass = \"IntervalTier\"\n\t\tname = \"John\"\n\t\txmin = 0\n\t\txmax = 2.3\n\t\tintervals: size = 1\n\t\tintervals [1]:\n\t\t\txmin = 0\n\t\t\txmax = 2.3\n\t\t\ttext = \"daisy bell\"\n\titem [2]:\n\t\tclass = \"IntervalTier\"\n\t\tname = \"Kelly\"\n\t\txmin = 0\n\t\txmax = 2.3\n\t\tintervals: size = 1\n\t\tintervals [1]:\n\t\t\txmin = 0\n\t\t\txmax = 2.3\n\t\t\ttext = \"\"\n\titem [3]:\n\t\tclass = \"TextTier\"\n\t\tname = \"Bell\"\n\t\txmin = 0\n\t\txmax = 2.3\n\t\tpoints: size = 1\n\t\tpoints [1]:\n\t\t\tnumber = 1\n\t\t\tmark = \"give me your answer do\"\"\n";

    #[test]
    fn parse_textgrid_from_string() {
        let parsed_textgrid = parse_textgrid(TEXTGRID, false).unwrap();

        let tier = match &parsed_textgrid.tiers()[1] {
            crate::textgrid::Tier::IntervalTier(tier) => tier,
            crate::textgrid::Tier::PointTier(_) => panic!("Expected IntervalTier, got PointTier"),
        };

        assert_eq!(tier.name(), "Kelly");
    }

    #[test]
    fn parse_textgrid_from_path() {
        let parsed_textgrid = parse_textgrid("example/long.TextGrid", false).unwrap();

        let tier = match &parsed_textgrid.tiers()[1] {
            crate::textgrid::Tier::IntervalTier(tier) => tier,
            crate::textgrid::Tier::PointTier(_) => panic!("Expected IntervalTier, got PointTier"),
        };

        assert_eq!(tier.name(), "Kelly");
    }

    #[test]
    fn parse_textgrid_from_vector() {
        let textgrid_vector = TEXTGRID
            .split('\n')
            .map(std::string::ToString::to_string)
            .collect::<Vec<String>>();

        let parsed_textgrid = parse_textgrid(textgrid_vector, false).unwrap();

        let tier = match &parsed_textgrid.tiers()[1] {
            crate::textgrid::Tier::IntervalTier(tier) => tier,
            crate::textgrid::Tier::PointTier(_) => panic!("Expected IntervalTier, got PointTier"),
        };

        assert_eq!(tier.name(), "Kelly");
    }

    #[test]
    fn parse_textgrid_from_stream() {
        let textgrid_stream = TEXTGRID.as_bytes();

        let parsed_textgrid =
            parse_textgrid(Source::Stream(Box::new(textgrid_stream)), false).unwrap();

        let tier = match &parsed_textgrid.tiers()[1] {
            crate::textgrid::Tier::IntervalTier(tier) => tier,
            crate::textgrid::Tier::PointTier(_) => panic!("Expected IntervalTier, got PointTier"),
        };

        assert_eq!(tier.name(), "Kelly");
    }

    #[test]
    fn parse_textgrid_from_file() {
        let textgrid_file = std::fs::File::open("example/long.TextGrid").unwrap();

        let parsed_textgrid = parse_textgrid(textgrid_file, false).unwrap();

        let tier = match &parsed_textgrid.tiers()[1] {
            crate::textgrid::Tier::IntervalTier(tier) => tier,
            crate::textgrid::Tier::PointTier(_) => panic!("Expected IntervalTier, got PointTier"),
        };

        assert_eq!(tier.name(), "Kelly");
    }

    #[test]
    fn parse_textgrid_from_invalid_string() {
        let parsed_textgrid = parse_textgrid("invalid", false);

        assert!(parsed_textgrid.is_err());
    }

    #[test]
    fn verify_start_of_textgrid() {
        let mut textgrid_data = VecDeque::new();
        textgrid_data.push_back("ooTextFile".to_string());
        textgrid_data.push_back("TextGrid".to_string());

        let verified_textgrid_data = super::verify_start_of_textgrid(&mut textgrid_data);

        assert!(verified_textgrid_data.is_ok());
    }

    #[test]
    fn parse_tiers() {
        let mut tier_data = VecDeque::new();
        tier_data.push_back("3".to_string());
        tier_data.push_back("IntervalTier".to_string());
        tier_data.push_back("John".to_string());
        tier_data.push_back("0".to_string());
        tier_data.push_back("2.3".to_string());
        tier_data.push_back("1".to_string());
        tier_data.push_back("0".to_string());
        tier_data.push_back("2.3".to_string());
        tier_data.push_back("daisy bell".to_string());
        tier_data.push_back("IntervalTier".to_string());
        tier_data.push_back("Kelly".to_string());
        tier_data.push_back("0".to_string());
        tier_data.push_back("2.3".to_string());
        tier_data.push_back("1".to_string());
        tier_data.push_back("0".to_string());
        tier_data.push_back("2.3".to_string());
        tier_data.push_back(String::new());
        tier_data.push_back("TextTier".to_string());
        tier_data.push_back("Bell".to_string());
        tier_data.push_back("0".to_string());
        tier_data.push_back("2.3".to_string());
        tier_data.push_back("1".to_string());
        tier_data.push_back("1".to_string());
        tier_data.push_back("give me your answer do\"".to_string());

        let parsed_tiers = super::parse_tiers(&mut tier_data, 0.0, 2.3, false).unwrap();

        let tier = match &parsed_tiers[1] {
            crate::textgrid::Tier::IntervalTier(tier) => tier,
            crate::textgrid::Tier::PointTier(_) => panic!("Expected IntervalTier, got PointTier"),
        };

        assert_eq!(tier.name(), "Kelly");
    }

    #[test]
    fn parse_interval() {
        let mut interval_data = VecDeque::new();
        interval_data.push_back("0".to_string());
        interval_data.push_back("2.3".to_string());
        interval_data.push_back("daisy bell".to_string());

        let parsed_interval = super::parse_interval(&mut interval_data).unwrap();

        assert_eq!(parsed_interval.text(), "daisy bell");
    }

    #[test]
    fn parse_point() {
        let mut point_data = VecDeque::new();
        point_data.push_back("1".to_string());
        point_data.push_back("give me your answer do\"".to_string());

        let parsed_point = super::parse_point(&mut point_data).unwrap();

        assert_eq!(parsed_point.mark(), "give me your answer do\"");
    }
}
