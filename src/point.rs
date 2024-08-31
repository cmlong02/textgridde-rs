use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
};

use derive_more::Constructor;
use getset::{Getters, Setters};

/// A "point," used in Praat as a specific time marker with an associated label.
#[derive(Constructor, Debug, Default, Clone, Getters, Setters)]
pub struct Point {
    #[getset(get = "pub", set = "pub")]
    number: f64,
    #[getset(get = "pub", set = "pub")]
    mark: String,
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "Point:\t{}\t{}", self.number, self.mark)
    }
}

/// Represents a point tier in a `TextGrid`.
#[derive(Clone, Constructor, Debug, Default, Getters, Setters)]
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

    /// Pushes a point to the tier.
    /// Calls `reorder()` to ensure the points are sorted by their number after pushing.
    ///
    /// # Arguments
    ///
    /// * `point` - The point to push.
    /// * `warn` - Whether to warn if the point is outside the tier's bounds.
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

        self.reorder();
    }

    /// Pushes multiple points to the tier by calling `push_point()` for each point.
    /// Calls `reorder()` to ensure the points are sorted by their number after pushing.
    ///
    /// # Arguments
    ///
    /// * `points` - The points to push.
    /// * `warn` - Whether to warn if the point is outside the tier's bounds.
    pub fn push_points<W: Into<Option<bool>> + Copy>(&mut self, points: Vec<Point>, warn: W) {
        if warn.into().unwrap_or_default() {
            for point in &points {
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
        }

        self.points.extend(points);

        self.reorder();
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

    /// Reorders the points in the tier by their number.
    pub fn reorder(&mut self) {
        self.points
            .sort_by(|a, b| a.number.partial_cmp(&b.number).unwrap_or(Ordering::Equal));
    }
}

impl Display for Tier {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "PointTier {}:
                xmin:  {}
                xmax:  {}
                point count: {}",
            self.name,
            self.xmin,
            self.xmax,
            self.points.len()
        )
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod test_point {
    #[test]
    fn test_point() {
        use crate::point::Point;

        let point = Point::new(1.0, "test".to_string());
        assert_eq!(point.number(), &1.0);
        assert_eq!(point.mark(), "test");
        assert_eq!(point.to_string(), "Point:\t1\ttest\n");
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod test_point_tier {
    #[test]
    fn set_xmin() {
        use crate::point::Tier;

        let mut tier = Tier::new("test".to_string(), 0.0, 10.0, vec![]);
        tier.set_xmin(5.0, Some(true));
        assert_eq!(tier.xmin(), &5.0);
    }

    #[test]
    fn set_xmax() {
        use crate::point::Tier;

        let mut tier = Tier::new("test".to_string(), 0.0, 10.0, vec![]);
        tier.set_xmax(5.0, Some(true));
        assert_eq!(tier.xmax(), &5.0);
    }

    #[test]
    fn get_size() {
        use crate::point::Tier;

        let tier = Tier::new("test".to_string(), 0.0, 10.0, vec![]);
        assert_eq!(tier.get_size(), 0);
    }

    #[test]
    fn push_point() {
        use crate::point::{Point, Tier};

        let mut tier = Tier::new("test".to_string(), 0.0, 10.0, vec![]);
        tier.push_point(Point::new(5.0, "test".to_string()), true);
        assert_eq!(tier.get_size(), 1);
    }

    #[test]
    fn push_points() {
        use crate::point::{Point, Tier};

        let mut tier = Tier::new("test".to_string(), 0.0, 10.0, vec![]);
        tier.push_points(vec![Point::new(5.0, "test".to_string())], true);
        assert_eq!(tier.get_size(), 1);
    }

    #[test]
    fn check_overlaps() {
        use crate::point::{Point, Tier};

        let mut tier = Tier::new("test".to_string(), 0.0, 10.0, vec![]);
        tier.push_points(
            vec![
                Point::new(5.0, "test".to_string()),
                Point::new(5.0, "test".to_string()),
            ],
            true,
        );
        assert_eq!(tier.check_overlaps(), Some(vec![(0, 1), (1, 0)]));
    }

    #[test]
    fn reorder() {
        use crate::point::{Point, Tier};

        let mut tier = Tier::new(
            "test".to_string(),
            0.0,
            10.0,
            vec![
                Point::new(5.0, "test".to_string()),
                Point::new(3.0, "test".to_string()),
            ],
        );
        tier.reorder();
        assert_eq!(tier.points()[0].number(), &3.0);
        assert_eq!(tier.points()[1].number(), &5.0);
    }
}
