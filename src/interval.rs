use std::cmp::Ordering;

use derive_more::Constructor;
use getset::{Getters, Setters};

#[derive(Constructor, Clone, Debug, Getters, Setters)]
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

/// Represents an interval tier in a `TextGrid`.
#[derive(Constructor, Debug, Getters, Setters)]
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
    /// A vector of the indices of the overlapping intervals or `None` if there are no overlaps.
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
        self.reorder();

        if self.intervals.len() < 2 {
            return;
        }

        let mut processed_intervals: Vec<Interval> = Vec::with_capacity(self.intervals.len());

        if prefer_first.into().unwrap_or(true) {
            for window in self.intervals.windows(2) {
                let interval = &window[0];
                let next_interval = &window[1];

                #[allow(clippy::float_cmp)]
                if interval.xmax == next_interval.xmin {
                    processed_intervals.push(interval.clone());
                } else {
                    let mut processed_interval = interval.clone();
                    processed_interval.xmax = next_interval.xmin;
                    processed_intervals.push(processed_interval);
                }
            }

            processed_intervals.push(self.intervals.last().unwrap().clone());
        } else {
            for window in self.intervals.windows(2).rev() {
                let interval = &window[0];
                let next_interval = &window[1];
                processed_intervals.push(interval.clone());

                #[allow(clippy::float_cmp)]
                if interval.xmax != next_interval.xmin {
                    processed_intervals.push(Interval::new(
                        interval.xmax,
                        next_interval.xmin,
                        next_interval.text.clone(),
                    ));
                }
            }

            processed_intervals.push(self.intervals.first().unwrap().clone());
        }

        self.intervals = processed_intervals;
    }

    /// Fills gaps with an interval with the specified text.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to fill the gaps with.
    ///
    /// # Panics
    ///
    /// If the amount of intervals exceeds `isize::MAX`.
    pub fn fill_gaps(&mut self, text: &str) {
        self.reorder();

        let mut processed_intervals: Vec<Interval> = Vec::with_capacity(self.intervals.len());
        for window in self.intervals.windows(2) {
            let interval = &window[0];
            let next_interval = &window[1];

            #[allow(clippy::float_cmp)]
            if interval.xmax != next_interval.xmin {
                processed_intervals.push(Interval::new(
                    interval.xmax,
                    next_interval.xmin,
                    text.into(),
                ));
            }
        }
        processed_intervals.push(self.intervals.last().unwrap().clone());
        self.intervals = processed_intervals;
    }
}
