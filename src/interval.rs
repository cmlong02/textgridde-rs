use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
};

use derive_more::Constructor;
use getset::{Getters, Setters};

/// An "interval," used in Praat as a specific period of time with an associated label.
#[derive(Clone, Constructor, Debug, Default, Getters, Setters)]
pub struct Interval {
    #[getset(get = "pub")]
    xmin: f64,
    #[getset(get = "pub")]
    xmax: f64,
    #[getset(get = "pub", set = "pub")]
    text: String,
}

impl Interval {
    /// Returns the duration of the interval.
    #[must_use]
    pub fn get_duration(&self) -> f64 {
        self.xmax - self.xmin
    }

    /// Returns the midpoint of the interval.
    #[must_use]
    pub fn get_midpoint(&self) -> f64 {
        (self.xmin + self.xmax) / 2.0
    }

    /// Sets the xmin value of the interval.
    ///
    /// # Arguments
    ///
    /// * `xmin` - The xmin value to set.
    pub fn set_xmin(&mut self, xmin: f64) {
        self.xmin = xmin;
    }

    /// Sets the xmax value of the interval.
    ///
    /// # Arguments
    ///
    /// * `xmax` - The xmax value to set.
    pub fn set_xmax(&mut self, xmax: f64) {
        self.xmax = xmax;
    }
}

impl Display for Interval {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "Interval")?;
        writeln!(f, "    xmin = {}", self.xmin)?;
        writeln!(f, "    xmax = {}", self.xmax)?;
        writeln!(f, "    text = \"{}\"", self.text)?;
        Ok(())
    }
}

/// Represents an interval tier in a `TextGrid`.
#[derive(Clone, Constructor, Debug, Default, Getters, Setters)]
pub struct Tier {
    #[getset(get = "pub", set = "pub")]
    name: String,
    #[getset(get = "pub")]
    xmin: f64,
    #[getset(get = "pub")]
    xmax: f64,
    #[getset(get = "pub")]
    intervals: Vec<Interval>,
}

impl Tier {
    /// Sets the minimum x value for the interval tier.
    ///
    /// # Arguments
    ///
    /// * `xmin` - The minimum x value to set.
    /// * `warn` - If `true`, displays a warning if the minimum point of any interval is greater than `xmin`.
    pub fn set_xmin<T: Into<Option<bool>>>(&mut self, xmin: f64, warn: T) {
        if warn.into().unwrap_or_default() {
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

    /// Sets the maximum x value for the interval tier.
    ///
    /// # Arguments
    ///
    /// * `xmax` - The maximum x value to set.
    /// * `warn` - If `true`, displays a warning if the maximum point of any interval is less than `xmax`.
    pub fn set_xmax<W: Into<Option<bool>>>(&mut self, xmax: f64, warn: W) {
        if warn.into().unwrap_or_default() {
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

    /// Returns the number of intervals in the interval tier.
    #[must_use]
    pub fn get_size(&self) -> usize {
        self.intervals.len()
    }

    /// Pushes an interval to the interval tier.
    /// Calls `reorder()` to ensure the intervals are sorted by their minimum x value after pushing the interval.
    ///
    /// # Arguments
    ///
    /// * `interval` - The interval to push.
    /// * `warn` - If `Some(true)`, displays a warning if the minimum point of the interval is less than the minimum x value of the interval tier.
    pub fn push_interval<W: Into<Option<bool>>>(&mut self, interval: Interval, warn: W) {
        if warn.into().unwrap_or_default() && interval.xmin < self.xmin {
            eprintln!(
                "Warning: Tier `{}` has a minimum point of {} but the TextGrid has an xmin of {}",
                self.name, interval.xmin, self.xmin
            );
        }
        self.intervals.push(interval);

        self.reorder();
    }

    /// Pushes multiple intervals to the interval tier vector.
    /// Calls `reorder()` afterwards to ensure the intervals are sorted by their minimum x value after pushing the intervals.
    ///
    /// # Arguments
    ///
    /// * `intervals` - The intervals to push.
    /// * `warn` - If `Some(true)`, displays a warning if the minimum point of any interval is less than the minimum x value of the interval tier.
    pub fn push_intervals<W: Into<Option<bool>> + Copy>(
        &mut self,
        intervals: Vec<Interval>,
        warn: W,
    ) {
        for interval in &intervals {
            if warn.into().unwrap_or_default() {
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

        self.intervals.extend(intervals);

        self.reorder();
    }

    /// Sets the intervals of the interval tier.
    ///
    /// # Arguments
    ///
    /// * `intervals` - The intervals to set.
    /// * `warn` - If `Some(true)`, displays a warning if any interval's minimum point is less than the minimum x value of the interval tier or if any interval's maximum point is greater than the maximum x value of the interval tier.
    pub fn set_intervals<W: Into<Option<bool>>>(&mut self, intervals: Vec<Interval>, warn: W) {
        if warn.into().unwrap_or_default() {
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

    /// Sorts the intervals in the interval tier by their minimum x value.
    fn reorder(&mut self) {
        self.intervals
            .sort_by(|a, b| a.xmin.partial_cmp(&b.xmin).unwrap_or(Ordering::Equal));
    }

    /// Checks for overlaps in the interval tier.
    /// Calls `reorder` to ensure the intervals are sorted by their minimum x value before checking for overlaps.
    ///
    /// # Returns
    ///
    /// A vector of pairs of the overlapping intervals' indices or `None` if there are no overlaps.
    #[must_use]
    pub fn check_overlaps(&self) -> Option<Vec<(u64, u64)>> {
        let mut overlaps: Vec<(u64, u64)> = Vec::new();

        // iterate over each pair of intervals, checking to make sure the xmax of the first interval is perfectly equal to the xmin of the second interval
        for (i, window) in self.intervals.windows(2).enumerate() {
            let interval = &window[0];
            let next_interval = &window[1];

            #[allow(clippy::float_cmp)]
            if interval.xmax != next_interval.xmin {
                overlaps.push((i as u64, (i + 1) as u64));
            }
        }

        if overlaps.is_empty() {
            None
        } else {
            Some(overlaps)
        }
    }

    /// Fixes gaps/overlaps in the interval tier.
    /// Calls `reorder` to ensure the intervals are sorted by their minimum x value before fixing gaps/overlaps.
    ///
    /// # Arguments
    ///
    /// * `prefer_first` - `true` by default. If `true`, prefers the first interval in the case of a gap. If `false`, prefers the second interval in the case of a gap.
    ///
    /// # Panics
    ///
    /// If the amount of intervals exceeds `isize::MAX`.
    pub fn fix_boundaries<P: Into<Option<bool>> + Copy>(&mut self, prefer_first: P) {
        if self.intervals.len() < 2 {
            return;
        }

        self.reorder();

        // Iterate over each pair of intervals, checking to make sure the xmax of the first
        // interval is perfectly equal to the xmin of the second interval. If not, handle
        // the gap by either modifying the xmax of the first interval or the xmin of the
        // second interval, depending on the value of `prefer_first`.
        if prefer_first.into().unwrap_or(true) {
            // Iterate in reverse, so we can modify the less-preferred interval without
            // affecting the preferred interval
            for i in (1..self.intervals.len()).rev() {
                let prev_interval = self.intervals[i - 1].clone();
                let interval = &mut self.intervals[i];

                #[allow(clippy::float_cmp)]
                if interval.xmin != prev_interval.xmax {
                    interval.xmin = prev_interval.xmax;
                }
            }
        } else {
            for i in 0..self.intervals.len() - 1 {
                let next_interval = self.intervals[i + 1].clone();
                let interval = &mut self.intervals[i];

                #[allow(clippy::float_cmp)]
                if interval.xmax != next_interval.xmin {
                    interval.xmax = next_interval.xmin;
                }
            }
        }
    }

    /// Fills gaps within an `IntervalTier` with the specified text, ensuring no time period
    /// is left empty.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to fill the gaps with.
    ///
    /// # Panics
    ///
    /// If the amount of intervals exceeds `isize::MAX`.
    #[allow(clippy::float_cmp)]
    pub fn fill_gaps(&mut self, text: &str) {
        if self.intervals.len() < 2 {
            return;
        }

        self.reorder();

        let first_xmin = self.intervals.first().unwrap().xmin;
        if first_xmin != self.xmin {
            let new_interval = Interval::new(self.xmin, first_xmin, text.to_string());
            self.intervals.insert(0, new_interval);
        }

        let last_xmax = self.intervals.last().unwrap().xmax;
        if last_xmax != self.xmax {
            let new_interval = Interval::new(last_xmax, self.xmax, text.to_string());
            self.intervals.push(new_interval);
        }

        for (index, window) in self.intervals.clone().windows(2).enumerate() {
            let interval = &window[0];
            let next_interval = &window[1];

            #[allow(clippy::float_cmp)]
            if interval.xmax != next_interval.xmin {
                let new_interval =
                    Interval::new(interval.xmax, next_interval.xmin, text.to_string());
                self.intervals.insert(index + 1, new_interval);
            }
        }
    }
}

impl Display for Tier {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "IntervalTier {}:
                xmin:  {}
                xmax:  {}
                interval count: {}",
            self.name,
            self.xmin,
            self.xmax,
            self.intervals.len()
        )
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod test_interval_tier {
    use crate::interval::Interval;

    #[test]
    fn get_duration() {
        let interval = Interval::new(0.0, 2.3, "test".to_string());

        assert_eq!(interval.get_duration(), 2.3);
    }

    #[test]
    fn get_midpoint() {
        let interval = Interval::new(0.0, 2.3, "test".to_string());

        assert_eq!(interval.get_midpoint(), 1.15);
    }

    #[test]
    fn set_xmin() {
        let mut interval = Interval::new(0.0, 2.3, "test".to_string());

        interval.set_xmin(1.0);

        assert_eq!(interval.xmin, 1.0);
    }

    #[test]
    fn set_xmax() {
        let mut interval = Interval::new(0.0, 2.3, "test".to_string());

        interval.set_xmax(1.0);

        assert_eq!(interval.xmax, 1.0);
    }

    #[test]
    fn to_string() {
        let interval = Interval::new(0.0, 2.3, "test".to_string());

        assert_eq!(
            interval.to_string(),
            "Interval\n    xmin = 0\n    xmax = 2.3\n    text = \"test\"\n"
        );
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod test_tier {
    use crate::interval::{Interval, Tier};

    #[test]
    fn set_xmin() {
        let mut tier = Tier::new("test".to_string(), 0.0, 2.3, Vec::new());

        tier.set_xmin(1.0, Some(true));

        assert_eq!(tier.xmin, 1.0);
    }

    #[test]
    fn set_xmax() {
        let mut tier = Tier::new("test".to_string(), 0.0, 2.3, Vec::new());

        tier.set_xmax(1.0, Some(true));

        assert_eq!(tier.xmax, 1.0);
    }

    #[test]
    fn get_size() {
        let tier = Tier::new("test".to_string(), 0.0, 2.3, Vec::new());

        assert_eq!(tier.get_size(), 0);
    }

    #[test]
    fn push_interval() {
        let mut tier = Tier::new("test".to_string(), 0.0, 2.3, Vec::new());

        tier.push_interval(Interval::new(0.0, 1.0, "test".to_string()), Some(true));

        assert_eq!(tier.intervals.len(), 1);
    }

    #[test]
    fn push_intervals() {
        let mut tier = Tier::new("test".to_string(), 0.0, 2.3, Vec::new());

        tier.push_intervals(
            vec![
                Interval::new(0.0, 1.0, "test".to_string()),
                Interval::new(1.0, 2.0, "test".to_string()),
            ],
            Some(true),
        );

        assert_eq!(tier.intervals.len(), 2);
    }

    #[test]
    fn set_intervals() {
        let mut tier = Tier::new("test".to_string(), 0.0, 2.3, Vec::new());

        tier.set_intervals(
            vec![
                Interval::new(0.0, 1.0, "test".to_string()),
                Interval::new(1.0, 2.0, "test".to_string()),
            ],
            Some(true),
        );

        assert_eq!(tier.intervals.len(), 2);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn reorder() {
        let mut tier = Tier::new("test".to_string(), 0.0, 2.3, Vec::new());

        tier.push_intervals(
            vec![
                Interval::new(1.0, 2.0, "test".to_string()),
                Interval::new(0.0, 1.0, "test".to_string()),
            ],
            Some(true),
        );

        tier.reorder();

        assert_eq!(tier.intervals[0].xmin, 0.0);
        assert_eq!(tier.intervals[1].xmin, 1.0);
    }

    mod check_overlaps {
        use crate::{
            interval::{Interval, Tier as IntervalTier},
            textgrid::{TextGrid, Tier},
        };

        #[test]
        fn no_overlap() {
            let mut textgrid = TextGrid::new(0.0, 2.3, Vec::new(), "test".to_string());

            textgrid.push_tier(
                Tier::IntervalTier(IntervalTier::new(
                    "John".to_string(),
                    0.0,
                    2.3,
                    vec![
                        Interval::new(0.0, 1.5, "daisy bell".to_string()),
                        Interval::new(1.5, 2.3, "daisy bell".to_string()),
                    ],
                )),
                false,
            );

            let overlaps = textgrid.check_overlaps();

            assert!(overlaps.is_none());
        }

        #[test]
        fn overlap() {
            let mut textgrid = TextGrid::new(0.0, 2.3, Vec::new(), "test".to_string());

            textgrid.push_tier(
                Tier::IntervalTier(IntervalTier::new(
                    "John".to_string(),
                    0.0,
                    2.3,
                    vec![
                        Interval::new(0.0, 1.5, "daisy bell".to_string()),
                        Interval::new(1.0, 2.3, "daisy bell".to_string()),
                    ],
                )),
                false,
            );

            let overlaps = textgrid.check_overlaps().unwrap();

            assert_eq!(overlaps.len(), 1);
            assert_eq!(overlaps[0].0, "John");
            assert_eq!(overlaps[0].1, (0, 1));
        }
    }

    #[allow(clippy::float_cmp)]
    mod fix_boundaries {
        use crate::interval::{Interval, Tier};

        #[test]
        fn prefer_first() {
            let mut tier = Tier::new("test".to_string(), 0.0, 2.3, Vec::new());

            tier.push_intervals(
                vec![
                    Interval::new(0.0, 1.2, "daisy".to_string()),
                    Interval::new(1.0, 1.75, "bell".to_string()),
                    Interval::new(1.5, 2.5, "answer".to_string()),
                    Interval::new(2.0, 5.0, "do".to_string()),
                ],
                false,
            );

            tier.fix_boundaries(true);

            assert_eq!(tier.intervals()[0].xmin(), &0.0);
            assert_eq!(tier.intervals()[0].xmax(), &1.2);
            assert_eq!(tier.intervals()[1].xmin(), &1.2);
            assert_eq!(tier.intervals()[1].xmax(), &1.75);
            assert_eq!(tier.intervals()[2].xmin(), &1.75);
            assert_eq!(tier.intervals()[2].xmax(), &2.5);
            assert_eq!(tier.intervals()[3].xmin(), &2.5);
            assert_eq!(tier.intervals()[3].xmax(), &5.0);
        }

        #[test]
        fn prefer_last() {
            let mut tier = Tier::new("test".to_string(), 0.0, 2.3, Vec::new());

            tier.push_intervals(
                vec![
                    Interval::new(0.0, 1.2, "daisy".to_string()),
                    Interval::new(1.0, 1.75, "bell".to_string()),
                    Interval::new(1.5, 2.5, "answer".to_string()),
                    Interval::new(2.0, 5.0, "do".to_string()),
                ],
                false,
            );

            tier.fix_boundaries(false);

            assert_eq!(tier.intervals()[0].xmin(), &0.0);
            assert_eq!(tier.intervals()[0].xmax(), &1.0);
            assert_eq!(tier.intervals()[1].xmin(), &1.0);
            assert_eq!(tier.intervals()[1].xmax(), &1.5);
            assert_eq!(tier.intervals()[2].xmin(), &1.5);
            assert_eq!(tier.intervals()[2].xmax(), &2.0);
            assert_eq!(tier.intervals()[3].xmin(), &2.0);
            assert_eq!(tier.intervals()[3].xmax(), &5.0);
        }
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn fill_gaps() {
        let mut tier = Tier::new("test".to_string(), 0.0, 2.3, Vec::new());

        tier.push_intervals(
            vec![
                Interval::new(0.0, 1.2, "daisy".to_string()),
                Interval::new(1.5, 2.3, "bell".to_string()),
            ],
            false,
        );

        tier.fill_gaps("gap");

        assert_eq!(tier.intervals()[1].text(), "gap");
        assert_eq!(tier.intervals()[1].xmin(), &1.2);
        assert_eq!(tier.intervals()[1].xmax(), &1.5);
    }

    #[test]
    fn to_string() {
        let tier = Tier::new("test".to_string(), 0.0, 2.3, Vec::new());

        assert_eq!(
            tier.to_string(),
            "IntervalTier test:
                xmin:  0
                xmax:  2.3
                interval count: 0"
        );
    }
}
