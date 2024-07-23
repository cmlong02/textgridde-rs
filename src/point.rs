use std::cmp::Ordering;

use derive_more::Constructor;
use getset::{Getters, Setters};

#[derive(Constructor, Debug, Getters, Setters)]
pub struct Point {
    #[getset(get = "pub", set = "pub")]
    number: f64,
    #[getset(get = "pub", set = "pub")]
    mark: String,
}

/// Represents a point tier in a `TextGrid`.
#[derive(Constructor, Debug, Getters, Setters)]
pub struct Tier {
    #[getset(get = "pub", set = "pub")]
    name: String,
    #[getset(get = "pub")]
    xmin: f64,
    #[getset(get = "pub")]
    xmax: f64,
    #[getset(get = "pub")]
    points: Vec<Point>,
}

impl Tier {
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

    pub fn push_point<W: Into<Option<bool>> + Copy>(&mut self, point: Point, warn: W) {
        if warn.into().unwrap_or_default() {
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

    /// Checks for overlaps in the tier.
    ///
    /// # Returns
    ///
    /// A vector of the indices of the overlapping points or `None` if there are no overlaps.
    #[must_use]
    pub fn check_overlaps(&self) -> Option<Vec<(u64, u64)>> {
        let mut overlaps: Vec<(u64, u64)> = Vec::new();
        for (i, point) in self.points.iter().enumerate() {
            for (j, other_point) in self.points.iter().enumerate() {
                #[allow(clippy::float_cmp)]
                if i != j && point.number == other_point.number {
                    overlaps.push((i as u64, j as u64));
                }
            }
        }
        if overlaps.is_empty() {
            None
        } else {
            Some(overlaps)
        }
    }
}
