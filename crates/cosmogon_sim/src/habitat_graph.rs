//! Engine-independent recorded graph display. Standard display transforms never enter simulation state or physics.
use crate::{habitat_comparison::Alignment, habitat_factorial::Effect, habitat_recording::Metric};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scale {
    #[default]
    Linear,
    Log10,
    SignedLog,
}
impl Scale {
    pub fn label(self) -> &'static str {
        match self {
            Self::Linear => "Linear",
            Self::Log10 => "Log10 (positive)",
            Self::SignedLog => "Signed log",
        }
    }
    pub fn forward(self, v: f64, t: f64) -> Option<f64> {
        if !v.is_finite() {
            return None;
        }
        Some(match self {
            Self::Linear => v,
            Self::Log10 => {
                if v <= 0. {
                    return None;
                }
                v.log10()
            }
            Self::SignedLog => {
                let a = v.abs() / t;
                if a <= 1. {
                    v / t
                } else {
                    v.signum() * (1. + a.log10())
                }
            }
        })
    }
    pub fn inverse(self, v: f64, t: f64) -> f64 {
        match self {
            Self::Linear => v,
            Self::Log10 => 10_f64.powf(v),
            Self::SignedLog => {
                if v.abs() <= 1. {
                    v * t
                } else {
                    v.signum() * t * 10_f64.powf(v.abs() - 1.)
                }
            }
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeUnit {
    Seconds,
    #[default]
    Hours,
    Days,
}
impl TimeUnit {
    pub fn factor(self) -> f64 {
        match self {
            Self::Seconds => 1.,
            Self::Hours => 3600.,
            Self::Days => 86400.,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Seconds => "s",
            Self::Hours => "terrestrial hours",
            Self::Days => "terrestrial days",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    pub format: String,
    pub schema_version: u32,
    pub metric: String,
    pub effect: String,
    pub alignment: String,
    pub window: [f64; 2],
    pub scale: Scale,
    pub signed_threshold: f64,
    pub include_zero: bool,
    pub time_unit: TimeUnit,
}
impl Default for Layout {
    fn default() -> Self {
        Self {
            format: "cosmogon-habitat-graph-layout".into(),
            schema_version: 1,
            metric: Metric::default().id().into(),
            effect: Effect::default().id().into(),
            alignment: "elapsed".into(),
            window: [0., 1.],
            scale: Scale::Linear,
            signed_threshold: 1.,
            include_zero: false,
            time_unit: TimeUnit::Hours,
        }
    }
}
impl Layout {
    pub fn validate(&self) -> Result<(), String> {
        if self.format != "cosmogon-habitat-graph-layout" || self.schema_version != 1 {
            return Err("Unsupported habitat graph layout format/version".into());
        }
        Metric::parse(&self.metric)?;
        Effect::parse(&self.effect)?;
        Alignment::parse(&self.alignment)?;
        let [a, b] = self.window;
        if !a.is_finite() || !b.is_finite() || a < 0. || b > 1. || b - a + 1e-15 < 1e-6 {
            return Err("Graph window must span at least one millionth of the complete time extent, within 0–1".into());
        }
        if !self.signed_threshold.is_finite() || !(1e-12..=1e12).contains(&self.signed_threshold) {
            return Err("Signed log threshold must be finite and between 1e-12 and 1e12 in current metric units".into());
        }
        Ok(())
    }
    pub fn read(path: &Path) -> Result<Self, String> {
        let m = std::fs::metadata(path).map_err(|e| e.to_string())?;
        if !m.is_file() || m.len() > 65536 {
            return Err("Graph layout must be a regular file of at most 64 KiB".into());
        }
        let f = std::fs::File::open(path).map_err(|e| e.to_string())?;
        if !f.metadata().map_err(|e| e.to_string())?.is_file() {
            return Err("Graph layout must be a regular file".into());
        }
        let mut bytes = Vec::new();
        f.take(65537)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 65536 {
            return Err("Graph layout exceeds 64 KiB".into());
        }
        let d: Self = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        d.validate()?;
        Ok(d)
    }
    pub fn export(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| format!("Choose a new layout file; existing files are protected: {e}"))?;
        f.write_all(&bytes).map_err(|e| e.to_string())
    }
    pub fn pan(&mut self, fraction: f64) {
        if !fraction.is_finite() {
            return;
        }
        let width = self.window[1] - self.window[0];
        let start = (self.window[0] + width * fraction).clamp(0., 1. - width);
        self.window = [start, start + width];
    }
    pub fn zoom(&mut self, factor: f64, anchor: f64) {
        if !factor.is_finite() || factor <= 0. || !anchor.is_finite() {
            return;
        }
        let old = self.window[1] - self.window[0];
        let anchor = anchor.clamp(0., 1.);
        let width = (old * factor).clamp(1e-6, 1.);
        let fixed = self.window[0] + old * anchor;
        let start = (fixed - width * anchor).clamp(0., 1. - width);
        self.window = [start, start + width];
    }
    pub fn center(&mut self, coordinate: f64, extent: [f64; 2]) {
        if extent[1] <= extent[0] {
            return;
        }
        let width = self.window[1] - self.window[0];
        let middle = ((coordinate - extent[0]) / (extent[1] - extent[0])).clamp(0., 1.);
        let start = (middle - width / 2.).clamp(0., 1. - width);
        self.window = [start, start + width];
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct Datum {
    pub index: usize,
    pub coordinate_s: f64,
    pub value: Option<f64>,
}
#[derive(Debug, Serialize)]
pub struct Plot {
    pub extent_s: [f64; 2],
    pub window_s: [f64; 2],
    pub displayed: Vec<Vec<Datum>>,
    pub supported_count: usize,
    pub unavailable_count: usize,
    pub scale_omitted_count: usize,
    pub y_bounds: Option<[f64; 2]>,
}
impl Plot {
    pub fn new(series: &[Vec<Datum>], layout: &Layout) -> Self {
        let mut extent = [f64::INFINITY, f64::NEG_INFINITY];
        for p in series.iter().flatten() {
            extent[0] = extent[0].min(p.coordinate_s);
            extent[1] = extent[1].max(p.coordinate_s);
        }
        if !extent[0].is_finite() || !extent[1].is_finite() {
            extent = [0., 0.];
        }
        let duration = extent[1] - extent[0];
        let window = [
            extent[0] + duration * layout.window[0],
            extent[0] + duration * layout.window[1],
        ];
        let displayed = series
            .iter()
            .map(|s| {
                s.iter()
                    .filter(|p| p.coordinate_s >= window[0] && p.coordinate_s <= window[1])
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let mut supported = 0;
        let mut unavailable = 0;
        let mut omitted = 0;
        let mut low = f64::INFINITY;
        let mut high = f64::NEG_INFINITY;
        for p in displayed.iter().flatten() {
            match p.value {
                None => unavailable += 1,
                Some(v) => match layout.scale.forward(v, layout.signed_threshold) {
                    Some(y) => {
                        supported += 1;
                        low = low.min(y);
                        high = high.max(y);
                    }
                    None => omitted += 1,
                },
            }
        }
        let bounds = if supported == 0 {
            None
        } else {
            if layout.include_zero && layout.scale != Scale::Log10 {
                low = low.min(0.);
                high = high.max(0.);
            }
            let pad = if layout.scale == Scale::Linear {
                (if high > low {
                    high - low
                } else if high == 0.0 {
                    // A zero-only chart has no measured span: one metric unit is a display baseline.
                    1.0
                } else {
                    high.abs().max(low.abs()) * 0.01
                })
                .max(high.abs().max(low.abs()) * f64::EPSILON * 16.0)
                .max(1e-300)
                    * 0.05
            } else {
                (high - low).max(0.1) * 0.05
            };
            Some([low - pad, high + pad])
        };
        Self {
            extent_s: extent,
            window_s: window,
            displayed,
            supported_count: supported,
            unavailable_count: unavailable,
            scale_omitted_count: omitted,
            y_bounds: bounds,
        }
    }
    pub fn nearest(&self, target: f64) -> Option<usize> {
        self.displayed
            .first()?
            .iter()
            .min_by(|a, b| {
                (a.coordinate_s - target)
                    .abs()
                    .total_cmp(&(b.coordinate_s - target).abs())
            })
            .map(|p| p.index)
    }
    pub fn selected_visible(&self, index: usize) -> bool {
        self.displayed
            .first()
            .is_some_and(|s| s.iter().any(|p| p.index == index))
    }
}

/// Interval-aware original-unit labels keep distinct small-signal ticks distinct.
pub fn tick(value: f64, step: f64) -> String {
    if step == 0.0 {
        return value.to_string();
    }
    let step = step.abs().max(f64::MIN_POSITIVE);
    if value.abs() >= 1e5 || (value != 0.0 && value.abs() < 0.01) {
        let precision = (value.abs().max(step) / step)
            .log10()
            .ceil()
            .clamp(2.0, 14.0) as usize
            + 1;
        format!("{value:.precision$e}")
    } else {
        let precision = (-step.log10()).ceil().clamp(0.0, 13.0) as usize + 1;
        format!("{value:.precision$}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn small_changes_have_resolved_bounds_and_distinct_original_unit_ticks() {
        let values = [293.15, 293.150001];
        let series = vec![values
            .iter()
            .enumerate()
            .map(|(i, &v)| Datum {
                index: i,
                coordinate_s: i as f64,
                value: Some(v),
            })
            .collect()];
        let p = Plot::new(&series, &Layout::default());
        let [lo, hi] = p.y_bounds.unwrap();
        assert!(lo < values[0] && hi > values[1] && hi - lo < 0.000002);
        let labels: std::collections::BTreeSet<_> = (0..=4)
            .map(|i| tick(lo + (hi - lo) * i as f64 / 4., (hi - lo) / 4.))
            .collect();
        assert_eq!(labels.len(), 5);
        for v in [0., 293.15, -1e-20, 1e15] {
            let p = Plot::new(
                &[vec![Datum {
                    index: 0,
                    coordinate_s: 0.,
                    value: Some(v),
                }]],
                &Layout::default(),
            );
            let [lo, hi] = p.y_bounds.unwrap();
            assert!(lo.is_finite() && hi.is_finite() && hi > lo && lo < v && hi > v);
        }
        assert_ne!(tick(0.000001, 0.0000001), tick(0.0000011, 0.0000001));
        assert_eq!(tick(0., 0.), "0");
        let zero = Plot::new(
            &[vec![Datum {
                index: 0,
                coordinate_s: 0.,
                value: Some(0.),
            }]],
            &Layout::default(),
        );
        assert_eq!(zero.y_bounds, Some([-0.05, 0.05]));
    }
    #[test]
    fn shared_layout_keeps_exact_versioned_choices_and_refuses_unknown_inputs() {
        let d = Layout {
            metric: "temperature".into(),
            window: [0.2, 0.8],
            scale: Scale::SignedLog,
            signed_threshold: 1e-4,
            time_unit: TimeUnit::Days,
            ..Default::default()
        };
        let bytes = serde_json::to_vec(&d).unwrap();
        let read: Layout = serde_json::from_slice(&bytes).unwrap();
        read.validate().unwrap();
        assert_eq!(read, d);
        let mut json = serde_json::to_value(&d).unwrap();
        json["unknown"] = serde_json::Value::Bool(true);
        assert!(serde_json::from_value::<Layout>(json).is_err());
        let invalid = Layout {
            window: [0.5, 0.5],
            ..d
        };
        assert!(invalid.validate().is_err());
    }
}
