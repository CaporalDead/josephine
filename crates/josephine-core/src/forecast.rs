//! Deterministic trend forecasting — the "prévoyance" pillar.
//!
//! Fits a straight line to a metric's recent history (ordinary least squares)
//! and projects when it will reach a target: a full disk, exhausted inodes, a
//! worn-out SSD. This is plain arithmetic — reproducible, fully local, no
//! learned model — so it sits inside Joséphine's "deterministic observation
//! only" rule rather than against it.
//!
//! The guards matter as much as the maths: a forecast is only produced when
//! there are enough samples, the fit is good enough, the trend *still holds
//! over the recent stretch*, the trend actually heads toward the target, and
//! the target is near enough to be worth mentioning. Otherwise Joséphine stays
//! quiet — she does not manufacture worry.

/// Guards that stop a forecast from crying wolf.
#[derive(Debug, Clone, Copy)]
pub struct Guards {
    /// Minimum number of samples needed to fit a line at all.
    pub min_samples: usize,
    /// Minimum goodness-of-fit (R², 0..=1) before a trend is trusted.
    pub min_fit: f64,
    /// Only speak when the target is within this many days.
    pub horizon_days: f64,
}

/// A straight-line projection of a metric toward a target value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Forecast {
    /// Change per day (positive = rising).
    pub slope_per_day: f64,
    /// Goodness of fit, 0..=1.
    pub fit_r2: f64,
    /// The fitted value at the latest sample (roughly "now").
    pub value_now: f64,
    /// Days from the latest sample until `target` is reached.
    pub eta_days: f64,
}

/// The tail of the observed span that has to confirm the trend.
///
/// A straight line through a *step* fits beautifully: a one-off 40-point jump
/// — a big download, a restore from backup — scores R² ≈ 0.75 over a month and
/// projects a full disk within the day, from a series that has been perfectly
/// flat for two weeks. Least squares cannot tell a step from a slope, so the
/// recent stretch has to still be moving the same way before we say anything.
const RECENT_SPAN_FRACTION: f64 = 1.0 / 3.0;
/// How much of the overall slope the recent stretch has to keep. Below this the
/// trend has stalled, and its ETA is a leftover from history.
const RECENT_SLOPE_RATIO: f64 = 0.5;

/// A least-squares line through a set of points.
struct Line {
    slope: f64,
    intercept: f64,
    r2: f64,
}

/// Ordinary least squares. `None` when there's no spread in time or the series
/// is perfectly flat — either way there is no trend to read.
fn fit_line(points: &[(f64, f64)]) -> Option<Line> {
    if points.len() < 2 {
        return None;
    }
    let n = points.len() as f64;
    let mean_x = points.iter().map(|p| p.0).sum::<f64>() / n;
    let mean_y = points.iter().map(|p| p.1).sum::<f64>() / n;

    let (mut sxx, mut sxy, mut syy) = (0.0, 0.0, 0.0);
    for &(x, y) in points {
        let (dx, dy) = (x - mean_x, y - mean_y);
        sxx += dx * dx;
        sxy += dx * dy;
        syy += dy * dy;
    }
    if sxx <= f64::EPSILON || syy <= f64::EPSILON {
        return None;
    }

    let slope = sxy / sxx;
    Some(Line {
        slope,
        intercept: mean_y - slope * mean_x,
        r2: (sxy * sxy) / (sxx * syy),
    })
}

/// Fit `(day, value)` points and project them to `target`. `day` is any
/// consistent day-scaled x (e.g. days since the Unix epoch); the slope comes
/// out per day. Returns `None` unless every guard passes.
pub fn project(points: &[(f64, f64)], target: f64, guards: &Guards) -> Option<Forecast> {
    if points.len() < guards.min_samples.max(2) {
        return None;
    }

    let line = fit_line(points)?;
    if line.r2 < guards.min_fit {
        return None;
    }

    // Points arrive in time order, but don't rely on it for the tail window.
    let x_last = points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
    let x_first = points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let span = x_last - x_first;
    if !span.is_finite() || span <= 0.0 {
        return None;
    }

    // Is the trend still there, or are we reading the far side of a step?
    let cutoff = x_last - span * RECENT_SPAN_FRACTION;
    let recent: Vec<(f64, f64)> = points.iter().copied().filter(|p| p.0 >= cutoff).collect();
    let recent_line = fit_line(&recent)?;
    if recent_line.slope.signum() != line.slope.signum()
        || recent_line.slope.abs() < line.slope.abs() * RECENT_SLOPE_RATIO
    {
        return None;
    }

    let value_now = line.slope * x_last + line.intercept;
    let remaining = target - value_now;

    // The trend has to actually head toward the target.
    if line.slope == 0.0 || remaining.signum() != line.slope.signum() {
        return None;
    }

    let eta_days = remaining / line.slope;
    if !eta_days.is_finite() || eta_days <= 0.0 || eta_days > guards.horizon_days {
        return None;
    }

    Some(Forecast {
        slope_per_day: line.slope,
        fit_r2: line.r2,
        value_now,
        eta_days,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guards() -> Guards {
        Guards {
            min_samples: 4,
            min_fit: 0.5,
            horizon_days: 30.0,
        }
    }

    #[test]
    fn projects_a_clean_rising_trend() {
        // +10 per day from 50; at the last point (day 4) the fit reads 90, so
        // 100 is one day away.
        let points = [
            (0.0, 50.0),
            (1.0, 60.0),
            (2.0, 70.0),
            (3.0, 80.0),
            (4.0, 90.0),
        ];
        let f = project(&points, 100.0, &guards()).expect("a forecast");
        assert!((f.slope_per_day - 10.0).abs() < 1e-6);
        assert!((f.value_now - 90.0).abs() < 1e-6);
        assert!((f.eta_days - 1.0).abs() < 1e-6);
        assert!(f.fit_r2 > 0.99);
    }

    #[test]
    fn flat_series_yields_nothing() {
        let points = [
            (0.0, 50.0),
            (1.0, 50.0),
            (2.0, 50.0),
            (3.0, 50.0),
            (4.0, 50.0),
        ];
        assert!(project(&points, 100.0, &guards()).is_none());
    }

    #[test]
    fn noisy_scatter_below_min_fit_is_rejected() {
        // Zig-zag with no real trend: R² well under 0.5.
        let points = [
            (0.0, 50.0),
            (1.0, 20.0),
            (2.0, 60.0),
            (3.0, 25.0),
            (4.0, 55.0),
        ];
        assert!(project(&points, 100.0, &guards()).is_none());
    }

    #[test]
    fn trend_away_from_target_is_rejected() {
        // Falling, but the target is above: it will never get there.
        let points = [
            (0.0, 90.0),
            (1.0, 80.0),
            (2.0, 70.0),
            (3.0, 60.0),
            (4.0, 50.0),
        ];
        assert!(project(&points, 100.0, &guards()).is_none());
    }

    #[test]
    fn eta_beyond_horizon_is_rejected() {
        // +0.1 per day from 50 needs ~450 days to hit 95 — far past 30.
        let points: Vec<(f64, f64)> = (0..20).map(|i| (i as f64, 50.0 + 0.1 * i as f64)).collect();
        assert!(project(&points, 95.0, &guards()).is_none());
    }

    #[test]
    fn too_few_samples_is_rejected() {
        let points = [(0.0, 50.0), (1.0, 70.0)];
        assert!(project(&points, 100.0, &guards()).is_none());
    }

    /// The regression this guards. Two weeks at 50%, one big download, two
    /// weeks at 90%: least squares reads that as +2.0%/day with R² = 0.75 and
    /// puts the disk full inside a day — from a series that has not moved
    /// since. Nothing is filling up; there is nothing to say.
    #[test]
    fn a_step_is_not_a_trend() {
        let mut points: Vec<(f64, f64)> = (0..15).map(|d| (d as f64, 50.0)).collect();
        points.extend((15..30).map(|d| (d as f64, 90.0)));

        // The fit itself is "good" — it is the plateau that disqualifies it.
        let line = fit_line(&points).expect("a line");
        assert!(line.r2 > 0.5, "r2 was {}", line.r2);
        assert!(line.slope > 0.0);

        assert!(project(&points, 100.0, &guards()).is_none());
    }

    /// A rise that levels off has an ETA left over from history, not ahead.
    #[test]
    fn a_trend_that_flattens_out_is_rejected() {
        let mut points: Vec<(f64, f64)> =
            (0..20).map(|d| (d as f64, 50.0 + 2.0 * d as f64)).collect();
        points.extend((20..30).map(|d| (d as f64, 88.0)));
        assert!(project(&points, 100.0, &guards()).is_none());
    }

    /// The counterpart: a rise that is still going is still projected.
    #[test]
    fn a_sustained_rise_still_projects() {
        let points: Vec<(f64, f64)> = (0..30).map(|d| (d as f64, 50.0 + 1.5 * d as f64)).collect();
        let f = project(&points, 100.0, &guards()).expect("a forecast");
        assert!((f.slope_per_day - 1.5).abs() < 1e-6);
        assert!(f.eta_days > 0.0 && f.eta_days <= 30.0);
    }

    /// Speeding up is still a trend — and the whole-window slope keeps the ETA
    /// on the conservative side.
    #[test]
    fn an_accelerating_rise_is_kept() {
        let points: Vec<(f64, f64)> = (0..30)
            .map(|d| (d as f64, 50.0 + 0.4 * d as f64 + 0.02 * (d * d) as f64))
            .collect();
        assert!(project(&points, 100.0, &guards()).is_some());
    }
}
