use std::{
    fmt::Debug,
    fs::{self, File},
    io::{Read, Result, Write},
    path::PathBuf,
};

use derive_more::Constructor;
use getset::{Getters, Setters};

use crate::{interval::Tier as IntervalTier, parse_textgrid, point::Tier as PointTier};

#[derive(Debug)]
pub enum Tier {
    IntervalTier(IntervalTier),
    PointTier(PointTier),
}

/// Represents the output format for writing the `TextGrid` to a file.
#[derive(Copy, Clone)]
pub enum OutputFormat {
    Long,
    Short,
}

#[derive(Constructor, Debug, Getters, Setters)]
/// Represents a `TextGrid`, which is a data structure used in the linguistic research program Praat
/// to annotate speech data. It can support either
pub struct TextGrid {
    #[getset(get = "pub")]
    xmin: f64,
    #[getset(get = "pub")]
    xmax: f64,
    #[getset(get = "pub")]
    tiers: Vec<Tier>,
    #[getset(get = "pub", set = "pub")]
    name: String,
}

impl TextGrid {
    /// Returns the number of tiers in the `TextGrid`.
    #[must_use]
    pub fn get_size(&self) -> usize {
        self.tiers.len()
    }

    /// Sets the xmin time value of the whole `TextGrid` in seconds.
    ///
    /// # Arguments
    ///
    /// * `xmin` - The new xmin value.
    /// * `warn` - If Some(true), displays a warning if any tier has an xmin lesser than `xmin`.
    pub fn set_xmin<W: Into<Option<bool>>>(&mut self, xmin: f64, warn: W) {
        if xmin > self.xmax {
            if warn.into().unwrap_or_default() {
                eprintln!("Warning: xmin cannot be greater than xmax. Setting to xmax.");
            }
            self.xmin = self.xmax;
            return;
        } else if xmin < 0.0 {
            if warn.into().unwrap_or_default() {
                eprintln!("Warning: xmin cannot be less than 0.0. Setting to 0.0.");
            }
            self.xmin = 0.0;
            return;
        }

        if warn.into().unwrap_or_default() {
            for tier in &self.tiers {
                match tier {
                    Tier::IntervalTier(interval_tier) => {
                        if *interval_tier.xmin() < xmin {
                            eprintln!(
                                "Warning: Tier `{}` has a minimum point of {} but the TextGrid has an xmin of {}",
                                interval_tier.name(), interval_tier.xmin(), xmin
                            );
                        }
                    }
                    Tier::PointTier(point_tier) => {
                        if *point_tier.xmin() < xmin {
                            eprintln!(
                                "Warning: Tier `{}` has a minimum point of {} but the TextGrid has an xmin of {}",
                                point_tier.name(), point_tier.xmin(), xmin
                            );
                        }
                    }
                }
            }
        }

        self.xmin = xmin;
    }

    /// Sets the xmax time value of the whole `TextGrid` in seconds.
    ///
    /// # Arguments
    ///
    /// * `xmax` - The new xmax value.
    /// * `warn` - If Some(true), displays a warning if any tier has an xmax greater than `xmax`.
    pub fn set_xmax<W: Into<Option<bool>>>(&mut self, xmax: f64, warn: W) {
        if xmax < self.xmin {
            if warn.into().unwrap_or_default() {
                eprintln!("Warning: xmax cannot be less than xmin. Setting to xmin.");
            }
            self.xmax = self.xmin;
            return;
        } else if xmax < 0.0 {
            if warn.into().unwrap_or_default() {
                eprintln!("Warning: xmax cannot be less than 0.0. Setting to 0.0.");
            }
            self.xmax = 0.0;
            return;
        }

        if warn.into().unwrap_or_default() {
            for tier in &self.tiers {
                match tier {
                    Tier::IntervalTier(interval_tier) => {
                        if *interval_tier.xmax() > xmax {
                            eprintln!(
                                "Warning: Tier `{}` has a maximum point of {} but the TextGrid has an xmax of {}",
                                interval_tier.name(), interval_tier.xmax(), xmax
                            );
                        }
                    }
                    Tier::PointTier(point_tier) => {
                        if *point_tier.xmax() > xmax {
                            eprintln!(
                                "Warning: Tier `{}` has a maximum point of {} but the TextGrid has an xmax of {}",
                                point_tier.name(), point_tier.xmax(), xmax
                            );
                        }
                    }
                }
            }
        }

        self.xmax = xmax;
    }

    /// Pushes a new, user-made tier to the `TextGrid`.
    ///
    /// # Arguments
    ///
    /// * `tier` - The tier to be added.
    /// * `warn` - If Some(true), displays a warning if the tier has a minimum or maximum point
    ///            that is outside the range of the `TextGrid`.
    pub fn push_tier<W: Into<Option<bool>> + Copy>(&mut self, mut tier: Tier, warn: W) {
        let name = match &tier {
            Tier::IntervalTier(interval_tier) => interval_tier.name(),
            Tier::PointTier(point_tier) => point_tier.name(),
        };

        let mut increment = 0;
        let mut new_name = name.to_string();
        while self.get_tier(&new_name).is_some() {
            increment += 1;
            new_name = format!("{name}{increment}");
        }
        if increment > 0 && warn.into().unwrap_or_default() {
            eprintln!("Warning: Tier name `{name}` already exists. Renaming to `{new_name}`");
        }

        if warn.into().unwrap_or_default() {
            match tier {
                Tier::IntervalTier(ref mut interval_tier) => {
                    interval_tier.set_name(new_name);

                    if *interval_tier.xmin() < self.xmin {
                        eprintln!(
                            "Warning: Tier `{}` has a minimum point of {} but the TextGrid has an xmin of {}",
                            interval_tier.name(), interval_tier.xmin(), self.xmin
                        );
                    }
                    if *interval_tier.xmax() > self.xmax {
                        eprintln!(
                            "Warning: Tier `{}` has a maximum point of {} but the TextGrid has an xmax of {}",
                            interval_tier.name(), interval_tier.xmax(), self.xmax
                        );
                    }
                }
                Tier::PointTier(ref mut point_tier) => {
                    point_tier.set_name(new_name);

                    if *point_tier.xmin() < self.xmin {
                        eprintln!(
                            "Warning: Tier `{}` has a minimum point of {} but the TextGrid has an xmin of {}",
                            point_tier.name(), point_tier.xmin(), self.xmin
                        );
                    }
                    if *point_tier.xmax() > self.xmax {
                        eprintln!(
                            "Warning: Tier `{}` has a maximum point of {} but the TextGrid has an xmax of {}",
                            point_tier.name(), point_tier.xmax(), self.xmax
                        );
                    }
                }
            }
        }

        self.tiers.push(tier);
    }

    /// Gets a tier using it's name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tier to get.
    ///
    /// # Returns
    ///
    /// Returns the tier if it exists, otherwise None.
    #[must_use]
    pub fn get_tier(&self, name: &str) -> Option<&Tier> {
        self.tiers.iter().find(|tier| match tier {
            Tier::IntervalTier(interval_tier) => interval_tier.name() == name,
            Tier::PointTier(point_tier) => point_tier.name() == name,
        })
    }

    /// Writes the `TextGrid` to a file or folder in the specified format.
    ///
    /// If given a folder path, the `TextGrid` will be written to a file in the folder with the same name as the `TextGrid`'s name field.
    ///
    /// Long `TextGrid`s are the typical format, while short
    /// `TextGrid`s are readable by Praat and do not include
    /// extraneous data.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file.
    /// * `format` - The output format.
    ///
    /// # Errors
    ///
    /// Returns an error if there was a problem creating or writing to the file.
    pub fn write(&self, path: PathBuf, format: OutputFormat) -> Result<()> {
        let mut file: File;
        fs::create_dir_all(path.clone())?;

        if path.is_dir() {
            let _ = fs::create_dir(path.clone());

            let mut path = path;
            path.push(format!("{}.TextGrid", self.name));

            file = File::create(path)?;
        } else {
            file = File::create(path)?;
        }

        let textgrid_data = match format {
            OutputFormat::Long => self.format_as_long(),
            OutputFormat::Short => self.format_as_short(),
        };

        file.write_all(textgrid_data.join("\n").as_bytes())?;

        Ok(())
    }

    /// Outputs a String vector containing the `TextGrid` to a file in the long format.
    fn format_as_long(&self) -> Vec<String> {
        let mut out_strings: Vec<String> = vec![
            "File type = \"ooTextFile\"".into(),
            "Object class = \"TextGrid\"".into(),
            String::new(),
            format!("xmin = {}", self.xmin),
            format!("xmax = {}", self.xmax),
            "tiers? <exists>".into(),
            format!("size = {}", self.tiers.len()),
            "item []:".into(),
        ];

        for (tier_index, tier) in self.tiers.iter().enumerate() {
            match tier {
                Tier::IntervalTier(interval_tier) => {
                    out_strings.push(format!("\titem [{}]:", tier_index + 1));
                    out_strings.push("\t\tclass = \"IntervalTier\"".into());
                    out_strings.push(format!("\t\tname = \"{}\"", interval_tier.name()));
                    out_strings.push(format!("\t\txmin = {}", interval_tier.xmin()));
                    out_strings.push(format!("\t\txmax = {}", interval_tier.xmax()));
                    out_strings.push(format!(
                        "\t\tintervals: size = {}",
                        interval_tier.get_size()
                    ));

                    for (interval_index, interval) in interval_tier.intervals().iter().enumerate() {
                        out_strings.push(format!("\t\tintervals [{}]:", interval_index + 1));
                        out_strings.push(format!("\t\t\txmin = {}", interval.xmin()));
                        out_strings.push(format!("\t\t\txmax = {}", interval.xmax()));
                        out_strings.push(format!("\t\t\ttext = \"{}\"", interval.text()));
                    }
                }
                Tier::PointTier(point_tier) => {
                    out_strings.push(format!("\titem [{}]:", tier_index + 1));
                    out_strings.push("\t\tclass = \"TextTier\"".into());
                    out_strings.push(format!("\t\tname = \"{}\"", point_tier.name()));
                    out_strings.push(format!("\t\txmin = {}", point_tier.xmin()));
                    out_strings.push(format!("\t\txmax = {}", point_tier.xmax()));
                    out_strings.push(format!("\t\tpoints: size = {}", point_tier.get_size()));

                    for (point_index, point) in point_tier.points().iter().enumerate() {
                        out_strings.push(format!("\t\tpoints [{}]:", point_index + 1));
                        out_strings.push(format!("\t\t\tnumber = {}", point.number()));
                        out_strings.push(format!("\t\t\tmark = \"{}\"", point.mark()));
                    }
                }
            }
        }

        out_strings
    }

    /// Outputs a String vector containing the `TextGrid` to a file in the short format.
    fn format_as_short(&self) -> Vec<String> {
        let mut out_strings: Vec<String> = vec![
            "\"ooTextFile\"".into(),
            "\"TextGrid\"".into(),
            String::new(),
            self.xmin.to_string(),
            self.xmax.to_string(),
            "<exists>".into(),
            self.tiers.len().to_string(),
        ];

        for tier in &self.tiers {
            match tier {
                Tier::IntervalTier(interval_tier) => {
                    out_strings.push("\"IntervalTier\"".into());
                    out_strings.push(format!("\"{}\"", interval_tier.name()));
                    out_strings.push(interval_tier.xmin().to_string());
                    out_strings.push(interval_tier.xmax().to_string());

                    for interval in interval_tier.intervals() {
                        out_strings.push(interval.xmin().to_string());
                        out_strings.push(interval.xmax().to_string());
                        out_strings.push(format!("\"{}\"", interval.text()));
                    }
                }
                Tier::PointTier(point_tier) => {
                    out_strings.push("\"TextTier\"".into());
                    out_strings.push(format!("\"{}\"", point_tier.name()));
                    out_strings.push(point_tier.xmin().to_string());
                    out_strings.push(point_tier.xmax().to_string());

                    for point in point_tier.points() {
                        out_strings.push(point.number().to_string());
                        out_strings.push(format!("\"{}\"", point.mark()));
                    }
                }
            }
        }

        out_strings
    }

    /// Checks the `TextGrid` for overlapping intervals or duplicate points.
    ///
    /// # Returns
    ///
    /// Returns Some([`tier_name`, (`index1`, `index2`)]) if an overlapping interval or point is found, otherwise None.
    #[must_use]
    pub fn check_overlaps(&self) -> Option<Vec<(String, (u64, u64))>> {
        let mut overlaps: Vec<(String, (u64, u64))> = Vec::new();

        for tier in &self.tiers {
            match tier {
                Tier::IntervalTier(interval_tier) => {
                    if let Some(interval_overlaps) = interval_tier.check_overlaps() {
                        overlaps.append(
                            &mut interval_overlaps
                                .into_iter()
                                .map(|overlap| (interval_tier.name().into(), overlap))
                                .collect(),
                        );
                    }
                }
                Tier::PointTier(point_tier) => {
                    if let Some(point_overlaps) = point_tier.check_overlaps() {
                        overlaps.append(
                            &mut point_overlaps
                                .into_iter()
                                .map(|overlap| (point_tier.name().into(), overlap))
                                .collect(),
                        );
                    }
                }
            }
        }

        if overlaps.is_empty() {
            None
        } else {
            Some(overlaps)
        }
    }

    /// Calls `fix_overlaps` on all interval tiers in the `TextGrid`.
    ///
    /// # Arguments
    ///
    /// * `prefer_first` - If true, the first interval's `xmax` will be raised to the new interval's `xmin` in the case of a gap or overlap.
    pub fn fix_boundaries<P: Into<Option<bool>> + Copy>(&mut self, prefer_first: bool) {
        for tier in &mut self.tiers {
            match tier {
                Tier::IntervalTier(interval_tier) => {
                    interval_tier.fix_boundaries(prefer_first);
                }
                Tier::PointTier(_) => {}
            }
        }
    }

    /// Calls `fill_gaps` on all interval tiers in the `TextGrid`.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to fill the gaps with.
    pub fn fill_gaps(&mut self, text: &str) {
        for tier in &mut self.tiers {
            match tier {
                Tier::IntervalTier(interval_tier) => {
                    interval_tier.fill_gaps(text);
                }
                Tier::PointTier(_) => {}
            }
        }
    }
}

/// `TextGrid::try_from` implementation for `PathBuf`.
impl TryFrom<PathBuf> for TextGrid {
    type Error = std::io::Error;

    fn try_from(path: PathBuf) -> Result<Self> {
        parse_textgrid(path, None)
    }
}

/// `TextGrid::try_from` implementation for `&str`.
impl TryFrom<&str> for TextGrid {
    type Error = std::io::Error;

    fn try_from(textgrid: &str) -> Result<Self> {
        parse_textgrid(textgrid, None)
    }
}

/// `TextGrid::try_from` implementation for `String`.
impl TryFrom<String> for TextGrid {
    type Error = std::io::Error;

    fn try_from(textgrid: String) -> Result<Self> {
        parse_textgrid(textgrid, None)
    }
}

impl TryFrom<Vec<String>> for TextGrid {
    type Error = std::io::Error;

    fn try_from(textgrid: Vec<String>) -> Result<Self> {
        parse_textgrid(textgrid, None)
    }
}

impl TryFrom<Box<dyn Read>> for TextGrid {
    type Error = std::io::Error;

    fn try_from(textgrid: Box<dyn Read>) -> Result<Self> {
        parse_textgrid(textgrid, None)
    }
}

impl TryFrom<File> for TextGrid {
    type Error = std::io::Error;

    fn try_from(textgrid: File) -> Result<Self> {
        parse_textgrid(textgrid, None)
    }
}
