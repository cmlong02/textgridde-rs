use std::{
    cmp::Ordering,
    io::{Read, Result},
};

use derive_more::Constructor;
use getset::{Getters, Setters};

use crate::parse_textgrid;

#[derive(Constructor, Getters, Setters)]
pub struct TextGrid {
    #[getset(get = "pub")]
    xmin: f64,
    #[getset(get = "pub")]
    xmax: f64,
    #[getset(get = "pub")]
    tiers: Vec<Tier>,
}

impl TextGrid {
    #[must_use]
    pub fn get_size(&self) -> usize {
        self.tiers.len()
    }

    pub fn set_xmin(&mut self, xmin: f64, warn: bool) {
        if warn {
            for tier in &self.tiers {
                match tier {
                    Tier::IntervalTier(interval_tier) => {
                        if interval_tier.xmin < xmin {
                            eprintln!(
                                "Warning: Tier `{}` has a minimum point of {} but the TextGrid has an xmin of {}",
                                interval_tier.name, interval_tier.xmin, xmin
                            );
                        }
                    }
                    Tier::PointTier(point_tier) => {
                        if point_tier.xmin < xmin {
                            eprintln!(
                                "Warning: Tier `{}` has a minimum point of {} but the TextGrid has an xmin of {}",
                                point_tier.name, point_tier.xmin, xmin
                            );
                        }
                    }
                }
            }
        }

        self.xmin = xmin;
    }

    pub fn set_xmax(&mut self, xmax: f64, warn: bool) {
        if warn {
            for tier in &self.tiers {
                match tier {
                    Tier::IntervalTier(interval_tier) => {
                        if interval_tier.xmax > xmax {
                            eprintln!(
                                "Warning: Tier `{}` has a maximum point of {} but the TextGrid has an xmax of {}",
                                interval_tier.name, interval_tier.xmax, xmax
                            );
                        }
                    }
                    Tier::PointTier(point_tier) => {
                        if point_tier.xmax > xmax {
                            eprintln!(
                                "Warning: Tier `{}` has a maximum point of {} but the TextGrid has an xmax of {}",
                                point_tier.name, point_tier.xmax, xmax
                            );
                        }
                    }
                }
            }
        }

        self.xmax = xmax;
    }

    pub fn push_tier(&mut self, tier: Tier, warn: Option<bool>) {
        if warn.is_some_and(|b| b) {
            match &tier {
                Tier::IntervalTier(interval_tier) => {
                    if interval_tier.xmin < self.xmin {
                        eprintln!(
                            "Warning: Tier `{}` has a minimum point of {} but the TextGrid has an xmin of {}",
                            interval_tier.name, interval_tier.xmin, self.xmin
                        );
                    }
                    if interval_tier.xmax > self.xmax {
                        eprintln!(
                            "Warning: Tier `{}` has a maximum point of {} but the TextGrid has an xmax of {}",
                            interval_tier.name, interval_tier.xmax, self.xmax
                        );
                    }
                }
                Tier::PointTier(point_tier) => {
                    if point_tier.xmin < self.xmin {
                        eprintln!(
                            "Warning: Tier `{}` has a minimum point of {} but the TextGrid has an xmin of {}",
                            point_tier.name, point_tier.xmin, self.xmin
                        );
                    }
                    if point_tier.xmax > self.xmax {
                        eprintln!(
                            "Warning: Tier `{}` has a maximum point of {} but the TextGrid has an xmax of {}",
                            point_tier.name, point_tier.xmax, self.xmax
                        );
                    }
                }
            }
        }

        self.tiers.push(tier);
    }

    enum OutputFormat {
        Long,
        Short,
    }

    pub fn write_to_file(&self, path: &PathBuf, format: OutputFormat) -> Result<()> {
        let mut file = File::create(path)?;

        match format {
            OutputFormat::Long => self.write_long(&mut file),
            OutputFormat::Short => self.write_short(&mut file),
        }
    }

    pub fn write_long(&self, file: &mut File) -> Result<()> {
        writeln!(file, "File type = \"ooTextFile\"")?;
        writeln!(file, "Object class = \"TextGrid\"")?;
        writeln!(file, "")?;
        writeln!(file, "xmin = {}", self.xmin)?;
        writeln!(file, "xmax = {}", self.xmax)?;
        writeln!(file, "tiers? <exists>")?;
        writeln!(file, "size = {}", self.get_size())?;

        for tier in &self.tiers {
            match tier {
                Tier::IntervalTier(interval_tier) => {
                    writeln!(file, "item []:")?;
                    writeln!(file, "    item [1]:")?;
                    writeln!(file, "        class = \"IntervalTier\"")?;
                    writeln!(file, "        name = \"{}\"", interval_tier.name)?;
                    writeln!(file, "        xmin = {}", interval_tier.xmin)?;
                    writeln!(file, "        xmax = {}", interval_tier.xmax)?;
                    writeln!(file, "        intervals: size = {}", interval_tier.get_size())?;

                    for interval in &interval_tier.intervals {
                        writeln!(file, "        intervals [{}]:", interval_tier.intervals.iter().position(|x| x == interval).unwrap_or_default() + 1)?;
                        writeln!(file, "            xmin = {}", interval.xmin)?;
                        writeln!(file, "            xmax = {}", interval.xmax)?;
                        writeln!(file, "            text = \"{}\"", interval.text)?;
                    }
                }
                Tier::PointTier(point_tier) => {
                    writeln!(file, "item []:")?;
                    writeln!(file, "    item [1]:")?;
                    writeln!(file, "        class = \"TextTier\"")?;
                    writeln!(file, "        name = \"{}\"", point_tier.name)?;
                    writeln!(file, "        xmin = {}", point_tier.xmin)?;
                    writeln!(file, "        xmax = {}", point_tier.xmax)?;
                    writeln!(file, "        points: size = {}", point_tier.get_size())?;

                    for point in &point_tier.points {
                        writeln!(file, "        points [{}]:", point_tier.points.iter().position(|x| x == point).unwrap_or_default() + 1)?;
                        writeln!(file, "            number = {}", point.number)?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn write_short(&self, file: &mut File) -> Result<()> {
        writeln!(file, "\"ooTextFile\"")?;
        writeln!(file, "\"TextGrid\"\n")?;
        
        writeln!(file, "{}", self.xmin)?;
        writeln!(file, "{}", self.xmax)?;
        writeln!(file, "<exists>")?;
        writeln!(file, "{}", self.get_size())?;

        for tier in &self.tiers {
            match tier {
                Tier::IntervalTier(interval_tier) => {
                    writeln!(file, "\"IntervalTier\"")?;
                    writeln!(file, "\"{}\"", interval_tier.name)?;
                    writeln!(file, "{}", interval_tier.xmin)?;
                    writeln!(file, "{}", interval_tier.xmax)?;
                    writeln!(file, "{}", interval_tier.get_size())?;

                    for interval in &interval_tier.intervals {
                        writeln!(file, "{}", interval.xmin)?;
                        writeln!(file, "{}", interval.xmax)?;
                        writeln!(file, "\"{}\"", interval.text)?;
                    }
                }
                Tier::PointTier(point_tier) => {
                    writeln!(file, "\"TextTier\"")?;
                    writeln!(file, "\"{}\"", point_tier.name)?;
                    writeln!(file, "{}", point_tier.xmin)?;
                    writeln!(file, "{}", point_tier.xmax)?;
                    writeln!(file, "{}", point_tier.get_size())?;

                    for point in &point_tier.points {
                        writeln!(file, "{}", point.number)?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl TryFrom<&str> for TextGrid {
    type Error = std::io::Error;

    fn try_from(textgrid: &str) -> Result<Self> {
        parse_textgrid(textgrid, None)
    }
}

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

pub enum Tier {
    IntervalTier(IntervalTier),
    PointTier(PointTier),
}

#[derive(Constructor, Getters, Setters)]
pub struct Interval {
    #[getset(get = "pub")]
    xmin: f64,
    #[getset(get = "pub")]
    xmax: f64,
    #[getset(get = "pub", set = "pub")]
    text: String,
}

#[derive(Constructor, Getters, Setters)]
pub struct IntervalTier {
    #[getset(get = "pub", set = "pub")]
    name: String,
    #[getset(get = "pub")]
    xmin: f64,
    #[getset(get = "pub")]
    xmax: f64,
    intervals: Vec<Interval>,
}

impl IntervalTier {
    pub fn set_xmin(&mut self, xmin: f64, warn: bool) {
        if warn {
            let min_point = self
                .intervals
                .iter()
                .filter_map(|intervals| {
                    intervals
                        .xmin
                        .partial_cmp(&f64::INFINITY)
                        .map(|_| intervals.xmin)
                })
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Greater)); // If invalid, return greater, since we're looking for the minimum

            if min_point.is_some_and(|min| xmin > min) {
                eprintln!("Warning: Tier `{}` has a minimum point of {} but the TextGrid has an xmin of {}", self.name, min_point.unwrap_or_default(), xmin);
            }
        }

        self.xmin = xmin;
    }

    pub fn set_xmax(&mut self, xmax: f64, warn: bool) {
        if warn {
            let max_point = self
                .intervals
                .iter()
                .filter_map(|interval| {
                    interval
                        .xmax
                        .partial_cmp(&f64::INFINITY)
                        .map(|_| interval.xmax)
                })
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Less)); // If invalid, return less, since we're looking for the maximum

            if max_point.is_some_and(|max| xmax < max) {
                eprintln!("Warning: Tier `{}` has a minimum point of {} but the TextGrid has an xmax of {}", self.name, max_point.unwrap_or_default(), xmax);
            }
        }

        self.xmax = xmax;
    }

    #[must_use]
    pub fn get_size(&self) -> usize {
        self.intervals.len()
    }

    pub fn push_interval(&mut self, interval: Interval, warn: Option<bool>) {
        if warn.is_some_and(|b| b) && interval.xmin < self.xmin {
            eprintln!(
                "Warning: Tier `{}` has a minimum point of {} but the TextGrid has an xmin of {}",
                self.name, interval.xmin, self.xmin
            );
        }
        self.intervals.push(interval);
    }

    pub fn set_intervals(&mut self, intervals: Vec<Interval>, warn: Option<bool>) {
        if warn.is_some_and(|b| b) {
            for interval in &intervals {
                if interval.xmin < self.xmin {
                    eprintln!(
                        "Warning: Tier `{}` has a minimum point of {} but the TextGrid has an xmin of {}",
                        self.name, interval.xmin, self.xmin
                    );
                }
                if interval.xmax > self.xmax {
                    eprintln!(
                        "Warning: Tier `{}` has a maximum point of {} but the TextGrid has an xmax of {}",
                        self.name, interval.xmax, self.xmax
                    );
                }
            }
        }

        self.intervals = intervals;
    }
}

#[derive(Constructor, Getters, Setters)]
pub struct Point {
    #[getset(get = "pub", set = "pub")]
    number: f64,
    #[getset(get = "pub", set = "pub")]
    mark: String,
}

#[derive(Constructor, Getters, Setters)]
pub struct PointTier {
    #[getset(get = "pub", set = "pub")]
    name: String,
    #[getset(get = "pub")]
    xmin: f64,
    #[getset(get = "pub")]
    xmax: f64,
    points: Vec<Point>,
}

impl PointTier {
    pub fn set_xmin(&mut self, xmin: f64, warn: Option<bool>) {
        if warn.is_some_and(|b| b) {
            let min_point = self
                .points
                .iter()
                .filter_map(|point| {
                    point
                        .number
                        .partial_cmp(&f64::INFINITY)
                        .map(|_| point.number)
                })
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Greater)); // If invalid, return greater, since we're looking for the minimum

            if min_point.is_some_and(|min| xmin > min) {
                eprintln!(
                    "Warning: Tier `{}` has a minimum point of {} but the set xmin is {}",
                    self.name,
                    min_point.unwrap_or_default(),
                    xmin
                );
            }
        }

        self.xmin = xmin;
    }

    pub fn set_xmax(&mut self, xmax: f64, warn: Option<bool>) {
        if warn.is_some_and(|b| b) {
            let max_point = self
                .points
                .iter()
                .filter_map(|point| {
                    point
                        .number
                        .partial_cmp(&f64::INFINITY)
                        .map(|_| point.number)
                })
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Less)); // If invalid, return less, since we're looking for the maximum

            if max_point.is_some_and(|max| xmax < max) {
                eprintln!(
                    "Warning: Tier `{}` has a maximum point of {} but the set xmax is {}",
                    self.name,
                    max_point.unwrap_or_default(),
                    xmax
                );
            }
        }

        self.xmax = xmax;
    }

    #[must_use]
    pub fn get_size(&self) -> usize {
        self.points.len()
    }

    #[must_use]
    pub const fn get_points(&self) -> &Vec<Point> {
        &self.points
    }

    pub fn push_point(&mut self, point: Point, warn: Option<bool>) {
        if warn.is_some_and(|b| b) {
            if point.number < self.xmin {
                eprintln!(
                    "Warning: Tier `{}` has a number of {} but the tier has an xmin of  {}",
                    self.name, point.number, self.xmin
                );
            }
            if point.number > self.xmax {
                eprintln!(
                    "Warning: Tier `{}` has a number of {} but the tier has an xmax of {}",
                    self.name, point.number, self.xmax
                );
            }
        }
        self.points.push(point);
    }
}
