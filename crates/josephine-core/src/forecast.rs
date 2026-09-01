//! Deterministic trend forecasting — the "prévoyance" pillar.
//!
//! Fits a straight line to a metric's recent history (ordinary least squares)
//! and projects when it will reach a target: a full disk, exhausted inodes, a
//! worn-out SSD. This is plain arithmetic — reproducible, fully local, no
//! learned model — so it sits inside Joséphine's "deterministic observation
//! only" rule rather than against it.
//!
//! The guards matter as much as the maths: a forecast is only produced when
//! there are enough samples, the fit is good enough, the trend actually heads
//! toward the target, and the target is near enough to be worth mentioning.
//! Otherwise Joséphine stays quiet — she does not manufacture worry.

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

/// Fit `(day, value)` points and project them to `target`. `day` is any
/// consistent day-scaled x (e.g. days since the Unix epoch); the slope comes
/// out per day. Returns `None` unless every guard passes.
pub fn project(points: &[(f64, f64)], target: f64, guards: &Guards) -> Option<Forecast> {
    if points.len() < guards.min_samples.max(2) {
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

    // No spread in time, or a perfectly flat series: no trend to project.
    if sxx <= f64::EPSILON || syy <= f64::EPSILON {
        return None;
    }

    let slope = sxy / sxx;
    let intercept = mean_y - slope * mean_x;
    let r2 = (sxy * sxy) / (sxx * syy);
    if r2 < guards.min_fit {
        return None;
    }

    let x_last = points.last()?.0;
    let value_now = slope * x_last + intercept;
    let remaining = target - value_now;

    // The trend has to actually head toward the target.
    if slope == 0.0 || remaining.signum() != slope.signum() {
        return None;
    }

    let eta_days = remaining / slope;
    if !eta_days.is_finite() || eta_days <= 0.0 || eta_days > guards.horizon_days {
        return None;
    }

    Some(Forecast {
        slope_per_day: slope,
        fit_r2: r2,
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
}
