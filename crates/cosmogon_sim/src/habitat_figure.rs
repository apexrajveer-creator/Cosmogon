//! Vector figures from actual stored observations. Display math never edits physics.
use crate::{
    habitat::Sample,
    habitat_comparison::Alignment,
    habitat_factorial::{Design, Effect},
    habitat_graph::{Datum, Layout, Plot, Scale},
    habitat_recording::{Metric, Recording},
};
use serde::Serialize;
use serde_json::{json, Value};
use std::{fmt::Write as _, io::Write, path::Path};
pub const LIMITS:&str="Stored model observations, not measured data or calibrated hardware. Markers are actual samples; connecting lines are visual guides, not reconstructed intermediate states. Unsupported observations and scale omissions break lines. No uncertainty bands, smoothing, interpolation or live-state substitution. Source identity and exact inputs are in figure.json; complete manifest is written last.";
#[derive(Clone, Debug, Serialize)]
pub struct Point {
    pub observation_index: usize,
    pub simulation_time_s: f64,
    pub completed_habitat_time_s: f64,
    pub completed_interval_s: f64,
    pub coordinate_s: f64,
    pub value: Option<f64>,
    pub support: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct Series {
    pub label: String,
    pub points: Vec<Point>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Figure {
    pub format: &'static str,
    pub schema_version: u32,
    pub layout: Layout,
    pub metric: &'static str,
    pub unit: &'static str,
    pub mode: &'static str,
    pub series: Vec<Series>,
    pub source_summaries: Vec<Value>,
    pub exact_samples: Vec<Vec<Sample>>,
    pub analysis: Option<Value>,
    pub configuration_assumptions: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heat_trajectory_context: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crop_trajectory_context: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_design_context: Option<Value>,
    pub limits: &'static str,
}
impl Figure {
    pub fn ordinary(a: &Recording, b: Option<&Recording>, layout: &Layout) -> Result<Self, String> {
        layout.validate()?;
        let metric = Metric::parse(&layout.metric)?;
        let alignment = Alignment::parse(&layout.alignment)?;
        let recordings: Vec<_> = std::iter::once(a).chain(b).collect();
        for r in &recordings {
            if r.rows.is_empty() || r.rows.len() > 1024 {
                return Err("Figures require 1–1024 actual observations per recording".into());
            }
        }
        let series = recordings
            .iter()
            .enumerate()
            .map(|(k, r)| Series {
                label: if k == 0 {
                    "A · recorded observations".into()
                } else {
                    "B · recorded observations".into()
                },
                points: r
                    .rows
                    .iter()
                    .enumerate()
                    .map(|(i, s)| {
                        let value = metric.value(s);
                        Point {
                            observation_index: i,
                            simulation_time_s: s.simulation_time_s,
                            completed_habitat_time_s: s.habitat.last_time,
                            completed_interval_s: s.habitat.budget.seconds,
                            coordinate_s: alignment
                                .coordinate(s.simulation_time_s, r.rows[0].simulation_time_s),
                            value,
                            support: if value.is_some()
                                && metric.is_heat()
                                && s.habitat.heat.as_ref().is_some_and(|h| {
                                    h.environment_missing_seconds
                                        + h.start_temperature_unsupported_seconds
                                        > 0.
                                }) {
                                "available_executed_subset"
                            } else if value.is_some() {
                                "available"
                            } else {
                                "unavailable"
                            }
                            .into(),
                        }
                    })
                    .collect(),
            })
            .collect();
        Ok(Self {
            format: "cosmogon-habitat-figure",
            schema_version: 1,
            layout: layout.clone(),
            metric: metric.label(),
            unit: metric.unit(),
            mode: "ordinary_observations",
            series,
            source_summaries: recordings.iter().map(|r| r.summary.clone()).collect(),
            exact_samples: recordings.iter().map(|r| r.rows.clone()).collect(),
            analysis: b.map(|b| crate::habitat_comparison::report(a, b, metric, alignment)),
            configuration_assumptions: None,
            heat_trajectory_context: metric
                .is_heat()
                .then(|| crate::habitat_heat_trajectory::context(&recordings)),
            crop_trajectory_context: metric.has_crop_context().then(|| crate::habitat_crop_trajectory::context(&recordings)),
            single_design_context: None,
            limits: LIMITS,
        })
    }
    pub fn paired(d: &Design, layout: &Layout) -> Result<Self, String> {
        layout.validate()?;
        let metric = Metric::parse(&layout.metric)?;
        let alignment = Alignment::parse(&layout.alignment)?;
        let effect = Effect::parse(&layout.effect)?;
        let plan = d.plan()?;
        let curve = d.curve(metric)?;
        let first = d.recordings[0]
            .rows
            .first()
            .ok_or("Empty paired recording")?
            .simulation_time_s;
        let points = curve
            .iter()
            .map(|p| Point {
                observation_index: p.observation_index,
                simulation_time_s: p.simulation_time_s,
                completed_habitat_time_s: d.recordings[0].rows[p.observation_index]
                    .habitat
                    .last_time,
                completed_interval_s: d.recordings[0].rows[p.observation_index]
                    .habitat
                    .budget
                    .seconds,
                coordinate_s: alignment.coordinate(p.simulation_time_s, first),
                value: effect.value(&p.contrasts),
                support: if effect.value(&p.contrasts).is_some() {
                    "matched"
                } else if metric.is_heat() || metric.is_crop() {
                    p.branch_support
                        .iter()
                        .copied()
                        .find(|s| *s != "matched")
                        .unwrap_or("unavailable_or_mismatched_interval")
                } else {
                    "unavailable_or_mismatched_interval"
                }
                .into(),
            })
            .collect();
        Ok(Self {
            format: "cosmogon-habitat-figure",
            schema_version: 1,
            layout: layout.clone(),
            metric: metric.label(),
            unit: metric.unit(),
            mode: "paired_contrast",
            series: vec![Series {
                label: effect.label().into(),
                points,
            }],
            source_summaries: d.recordings.iter().map(|r| r.summary.clone()).collect(),
            exact_samples: d.recordings.iter().map(|r| r.rows.clone()).collect(),
            analysis: Some(
                json!({"plan":plan,"common_state_fingerprint":d.manifest["common_state_fingerprint"],"common_data_versions":d.manifest["common_data_versions"],"final_analysis":d.analysis,"selected_metric_points":curve}),
            ),
            configuration_assumptions: plan.configuration_assumptions(),
            heat_trajectory_context: metric.is_heat().then(|| {
                crate::habitat_heat_trajectory::context(&d.recordings.iter().collect::<Vec<_>>())
            }),
            crop_trajectory_context: metric.has_crop_context().then(|| crate::habitat_crop_trajectory::context(&d.recordings.iter().collect::<Vec<_>>())),
            single_design_context: None,
            limits: LIMITS,
        })
    }
    pub fn read(
        recording: &Path,
        reference: Option<&Path>,
        layout: &Layout,
    ) -> Result<Self, String> {
        if recording.join("plan.json").exists() {
            if reference.is_some() {
                return Err("Design figures use their retained branches; an extra reference is not supported".into());
            }
            match crate::habitat_design::read(recording)? {
                crate::habitat_design::Design::Paired(d) => Self::paired(&d,layout),
                crate::habitat_design::Design::Single(d) => d.figure(0,d.recordings.len()-1,layout),
            }
        } else {
            let a = Recording::read(recording)?;
            let b = reference.map(Recording::read).transpose()?;
            Self::ordinary(&a, b.as_ref(), layout)
        }
    }
    pub fn plot(&self) -> Plot {
        Plot::new(
            &self
                .series
                .iter()
                .map(|s| {
                    s.points
                        .iter()
                        .map(|p| Datum {
                            index: p.observation_index,
                            coordinate_s: p.coordinate_s,
                            value: p.value,
                        })
                        .collect()
                })
                .collect::<Vec<_>>(),
            &self.layout,
        )
    }
    fn visibility(&self, p: &Point, plot: &Plot) -> &'static str {
        if p.coordinate_s < plot.window_s[0] || p.coordinate_s > plot.window_s[1] {
            "outside_window"
        } else if p.value.is_none() {
            "unavailable"
        } else if p
            .value
            .and_then(|v| self.layout.scale.forward(v, self.layout.signed_threshold))
            .is_none()
        {
            "scale_omitted"
        } else {
            "displayed"
        }
    }
    pub fn csv(&self) -> String {
        let plot = self.plot();
        let mut out="series,observation_index,simulation_time_s,completed_habitat_time_s,completed_interval_s,coordinate_s,value,unit,support,display_state\n".to_string();
        for (i, s) in self.series.iter().enumerate() {
            for p in &s.points {
                let value = p.value.map(|v| format!("{v:.17e}")).unwrap_or_default();
                let _ = writeln!(
                    out,
                    "{i},{},{:.17e},{:.17e},{:.17e},{:.17e},{},{},{},{}",
                    p.observation_index,
                    p.simulation_time_s,
                    p.completed_habitat_time_s,
                    p.completed_interval_s,
                    p.coordinate_s,
                    value,
                    self.unit,
                    p.support,
                    self.visibility(p, &plot)
                );
            }
        }
        out
    }
    pub fn svg(&self) -> String {
        let plot = self.plot();
        let [x0, x1] = plot.window_s;
        let unit = self.layout.time_unit.factor();
        let x = |t: f64| {
            if x1 > x0 {
                180.0 + (t - x0) / (x1 - x0) * 730.0
            } else {
                545.0
            }
        };
        let mut out=String::from("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1000\" height=\"720\" viewBox=\"0 0 1000 720\" role=\"img\" aria-labelledby=\"title desc\"><title id=\"title\">");
        out.push_str(&xml(self.metric));
        out.push_str("</title><desc id=\"desc\">");
        out.push_str(&xml(LIMITS));
        if self.heat_trajectory_context.is_some() {
            out.push_str(&xml(crate::habitat_heat_trajectory::SCOPE));
        }
        if self.crop_trajectory_context.is_some() { out.push_str(&xml(crate::habitat_crop_trajectory::SCOPE)); }
        if self.single_design_context.is_some() {out.push_str(" Selected independent single-setting branches; typed setting/unit, complete plan and common source identity are in figure.json.");}
        out.push_str("</desc>");
        let mut metadata=json!({"format":self.format,"schema_version":self.schema_version,"layout":self.layout,"mode":self.mode,"source_summaries":self.source_summaries,"full_inputs":"figure.json","numeric_values":"points.csv","heat_trajectory_context":self.heat_trajectory_context});
        if let Some(c)=&self.crop_trajectory_context {metadata["crop_trajectory_context"]=c.clone();}
        if let Some(c)=&self.single_design_context {metadata["single_design_context"]=c.clone();}
        let _=write!(out,"<metadata>{}</metadata>",xml(&metadata.to_string()));
        out.push_str("<rect width=\"1000\" height=\"720\" fill=\"white\"/><g font-family=\"sans-serif\" fill=\"#18212c\">");
        text(&mut out, 110., 38., 24., "start", self.metric);
        text(
            &mut out,
            110.,
            66.,
            14.,
            "start",
            &format!(
                "{} · {} · {}",
                if self.mode == "paired_contrast" {
                    "Paired model contrast"
                } else {
                    "Recorded model observations"
                },
                self.layout.scale.label(),
                self.unit
            ),
        );
        out.push_str("<rect x=\"180\" y=\"110\" width=\"730\" height=\"390\" fill=\"none\" stroke=\"#58616e\"/>");
        for i in 0..=if x1 > x0 { 4 } else { 0 } {
            let t = x0 + (x1 - x0) * i as f64 / 4.;
            let xx = x(t);
            let _ = write!(
                out,
                "<path d=\"M{xx:.4} 110V500\" stroke=\"#e3e7eb\" fill=\"none\"/>"
            );
            text(
                &mut out,
                xx,
                525.,
                13.,
                "middle",
                &crate::habitat_graph::tick(t / unit, (x1 - x0) / unit / 4.),
            );
        }
        text(
            &mut out,
            545.,
            556.,
            15.,
            "middle",
            &format!(
                "{} ({})",
                if self.layout.alignment == "elapsed" {
                    "Elapsed from each recording's first observation"
                } else {
                    "Simulation world time"
                },
                self.layout.time_unit.label()
            ),
        );
        if let Some([lo, hi]) = plot.y_bounds {
            let y = |v: f64| 500.0 - (v - lo) / (hi - lo) * 390.0;
            for i in 0..=4 {
                let v = lo + (hi - lo) * i as f64 / 4.;
                let yy = y(v);
                let _ = write!(
                    out,
                    "<path d=\"M180 {yy:.4}H910\" stroke=\"#e3e7eb\" fill=\"none\"/>"
                );
                text(
                    &mut out,
                    168.,
                    yy + 4.,
                    13.,
                    "end",
                    &crate::habitat_graph::tick(
                        self.layout.scale.inverse(v, self.layout.signed_threshold),
                        (self.layout.scale.inverse(hi, self.layout.signed_threshold)
                            - self.layout.scale.inverse(lo, self.layout.signed_threshold))
                            / 4.,
                    ),
                );
            }
            for (k, s) in self.series.iter().enumerate() {
                let color = if k == 0 { "#0067a5" } else { "#b65010" };
                let mut previous = None;
                for p in &s.points {
                    if self.visibility(p, &plot) != "displayed" {
                        previous = None;
                        continue;
                    }
                    let v = self
                        .layout
                        .scale
                        .forward(p.value.unwrap(), self.layout.signed_threshold)
                        .unwrap();
                    let (xx, yy) = (x(p.coordinate_s), y(v));
                    if let Some((px, py)) = previous {
                        let _=write!(out,"<path d=\"M{px:.4} {py:.4}L{xx:.4} {yy:.4}\" stroke=\"{color}\" stroke-width=\"1.7\" {} fill=\"none\"/>",if k==0 {""}else{"stroke-dasharray=\"5 4\""});
                    }
                    let _ = if k == 0 {
                        write!(
                            out,
                            "<circle cx=\"{xx:.4}\" cy=\"{yy:.4}\" r=\"3\" fill=\"{color}\"/>"
                        )
                    } else {
                        write!(out,"<rect x=\"{:.4}\" y=\"{:.4}\" width=\"6\" height=\"6\" fill=\"{color}\"/>",xx-3.,yy-3.)
                    };
                    previous = Some((xx, yy));
                }
            }
        } else {
            text(
                &mut out,
                545.,
                310.,
                18.,
                "middle",
                if plot.scale_omitted_count > 0 {
                    "No values supported by this scale"
                } else if plot.displayed.iter().all(Vec::is_empty) {
                    "No stored observations in this time window"
                } else {
                    "No supported observations for this metric"
                },
            );
        }
        for (k, s) in self.series.iter().enumerate() {
            let xx = 110. + k as f64 * 400.;
            let color = if k == 0 { "#0067a5" } else { "#b65010" };
            let _ = write!(
                out,
                "<path d=\"M{xx} 88h24\" stroke=\"{color}\" stroke-width=\"3\" {} />",
                if k == 0 {
                    ""
                } else {
                    "stroke-dasharray=\"5 4\""
                }
            );
            text(&mut out, xx + 32., 93., 13., "start", &s.label);
        }
        text(
            &mut out,
            110.,
            585.,
            13.,
            "start",
            &format!(
                "{} displayed · {} unavailable · {} omitted by scale",
                plot.supported_count, plot.unavailable_count, plot.scale_omitted_count
            ),
        );
        if self.layout.scale == Scale::SignedLog {
            text(
                &mut out,
                110.,
                608.,
                13.,
                "start",
                &format!(
                    "Signed-log threshold: {} {} · ticks retain original units",
                    number(self.layout.signed_threshold),
                    self.unit
                ),
            );
        }
        text(
            &mut out,
            110.,
            635.,
            13.,
            "start",
            if self.heat_trajectory_context.is_some() {
                "Cumulative from ledger origin; executed subset only. Coverage/origins: figure.json."
            } else if self.crop_trajectory_context.is_some() {
                "Finite reference crop; progress is not biological age. Assumptions: figure.json."
            } else {
                "Markers: stored samples. Lines: visual guides; gaps remain unavailable."
            },
        );
        text(&mut out,110.,658.,13.,"start","Source/model identity, exact inputs and assumptions: figure.json; original numbers: points.csv.");
        text(&mut out,110.,681.,13.,"start","Model sensitivity only; no measured-hardware accuracy or predictive uncertainty claimed.");
        if let Some(context)=&self.single_design_context {
            let name=context["manifest"]["parameter"].as_str().and_then(|id|crate::habitat_sweep::Parameter::parse(id).ok()).map(|p|p.label()).unwrap_or("saved setting");
            let unit=context["manifest"]["unit"].as_str().unwrap_or("");
            text(&mut out,110.,704.,12.,"start",&format!("Varied setting: {name} ({unit}); branch values are in the legend."));
        }
        out.push_str("</g></svg>\n");
        out
    }
    pub fn export(&self, path: &Path) -> Result<Value, String> {
        self.layout.validate()?;
        let json = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        if json.len() > 64 * 1024 * 1024 {
            return Err("Complete figure inputs exceed the 64 MiB export bound".into());
        }
        let plot = self.plot();
        let manifest = json!({"format":"cosmogon-habitat-figure-export","schema_version":1,"complete":true,"layout":self.layout,"mode":self.mode,"displayed":plot.supported_count,"unavailable":plot.unavailable_count,"scale_omitted":plot.scale_omitted_count,"files":["figure.svg","points.csv","figure.json","layout.json"],"limits":LIMITS});
        let files = [
            ("figure.svg", self.svg().into_bytes()),
            ("points.csv", self.csv().into_bytes()),
            ("figure.json", json),
            (
                "layout.json",
                serde_json::to_vec_pretty(&self.layout).map_err(|e| e.to_string())?,
            ),
            (
                "manifest.json",
                serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
            ),
        ];
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::create_dir(path)
            .map_err(|e| format!("Existing figures are protected; choose a new folder: {e}"))?;
        for (name, bytes) in files {
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path.join(name))
                .map_err(|e| format!("Incomplete figure: {e}"))?;
            f.write_all(&bytes)
                .map_err(|e| format!("Incomplete figure: {e}"))?;
        }
        Ok(manifest)
    }
}
fn number(v: f64) -> String {
    if v != 0. && (v.abs() < 1e-3 || v.abs() >= 1e5) {
        format!("{v:.3e}")
    } else {
        format!("{v:.3}")
    }
}
fn text(out: &mut String, x: f64, y: f64, size: f64, anchor: &str, s: &str) {
    let _ = write!(
        out,
        "<text x=\"{x}\" y=\"{y}\" font-size=\"{size}\" text-anchor=\"{anchor}\">{}</text>",
        xml(s)
    );
}
fn xml(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c if c < ' ' && !matches!(c, '\t' | '\n' | '\r') => out.push('\u{fffd}'),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        habitat::{Definition, Environment, Habitat},
        habitat_factorial::Secondary,
        habitat_sweep::{Parameter, Plan},
        BodyRef,
    };
    fn recording() -> Recording {
        let mut h = Habitat::new(
            0,
            Definition::at(BodyRef { system: 0, body: 0 }, 0., 0.),
            1000.,
        )
        .unwrap();
        let mut rows = Vec::new();
        for i in 0..4 {
            if i > 0 {
                h.step(
                    1000. + i as f64 * 60.,
                    Environment {
                        irradiance_w_m2: Some(0.),
                        exterior_temperature_k: Some(250.),
                    },
                )
                .unwrap();
            }
            rows.push(Sample {
                simulation_time_s: 1000. + i as f64 * 60.,
                habitat: h.clone(),
            });
        }
        Recording {
            rows,
            summary: json!({"name":"Hostile <script>&\"' cabin","source":"actual retained identity"}),
        }
    }
    #[test]
    fn ordinary_keeps_every_source_value_and_independent_time_alignment() {
        let a = recording();
        let mut b = a.clone();
        for s in &mut b.rows {
            s.simulation_time_s += 100.;
            s.habitat.last_time += 100.;
            s.habitat.state.temperature_k += 2.;
        }
        let original = serde_json::to_vec(&a.rows).unwrap();
        let layout = Layout {
            metric: "temperature".into(),
            time_unit: crate::habitat_graph::TimeUnit::Seconds,
            ..Default::default()
        };
        let f = Figure::ordinary(&a, Some(&b), &layout).unwrap();
        assert_eq!(f.exact_samples, vec![a.rows.clone(), b.rows.clone()]);
        assert_eq!(
            f.source_summaries,
            vec![a.summary.clone(), b.summary.clone()]
        );
        for (i, s) in f.series.iter().enumerate() {
            for (j, p) in s.points.iter().enumerate() {
                let raw = &f.exact_samples[i][j];
                assert_eq!(p.simulation_time_s, raw.simulation_time_s);
                assert_eq!(p.completed_habitat_time_s, raw.habitat.last_time);
                assert_eq!(p.completed_interval_s, raw.habitat.budget.seconds);
                assert_eq!(p.coordinate_s, j as f64 * 60.);
                assert_eq!(p.value, Some(raw.habitat.state.temperature_k));
            }
        }
        assert_eq!(
            f.svg(),
            Figure::ordinary(&a, Some(&b), &layout).unwrap().svg()
        );
        assert_eq!(f.csv().lines().count(), 9);
        assert!(f.csv().lines().all(|r| r.split(',').count() == 10));
        let world = Layout {
            alignment: "world".into(),
            ..layout
        };
        let f = Figure::ordinary(&a, Some(&b), &world).unwrap();
        assert_eq!(f.series[1].points[0].coordinate_s, 1100.);
        assert!(f.analysis.is_some());
        assert_eq!(serde_json::to_vec(&a.rows).unwrap(), original);
        assert!(!f.svg().contains("<script>"));
        assert!(f.svg().contains("&lt;script&gt;&amp;"));
    }
    #[test]
    fn unavailable_and_nonpositive_scale_points_break_paths_without_changing_numbers() {
        let mut r = recording();
        r.rows[2].habitat.budget.environment_available = false;
        let f = Figure::ordinary(
            &r,
            None,
            &Layout {
                metric: "delivered_power".into(),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(f.plot().unavailable_count, 2);
        assert_eq!(f.csv().matches(",unavailable,unavailable").count(), 2);
        assert!(!f.svg().contains("stroke-width=\"1.7\""));
        for (i, s) in r.rows.iter_mut().enumerate() {
            s.habitat.state.temperature_k = [2., 0., -2., 4.][i];
        }
        let layout = Layout {
            metric: "temperature".into(),
            scale: Scale::Log10,
            ..Default::default()
        };
        let log = Figure::ordinary(&r, None, &layout).unwrap();
        assert_eq!(log.plot().scale_omitted_count, 2);
        assert_eq!(log.plot().unavailable_count, 0);
        assert_eq!(log.csv().matches(",scale_omitted").count(), 2);
        assert!(!log.svg().contains("stroke-width=\"1.7\""));
        assert_eq!(log.series[0].points[2].value, Some(-2.));
        let signed = Figure::ordinary(
            &r,
            None,
            &Layout {
                scale: Scale::SignedLog,
                ..layout
            },
        )
        .unwrap();
        assert_eq!(signed.plot().supported_count, 4);
        assert!(signed.svg().contains("Signed-log threshold"));
        assert_eq!(signed.svg().matches("stroke-width=\"1.7\"").count(), 3);
        let missing = Figure::ordinary(
            &r,
            None,
            &Layout {
                metric: "vapor".into(),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(missing.plot().y_bounds.is_none());
        assert!(missing.svg().contains("No supported observations"));
        let empty = Figure::ordinary(
            &r,
            None,
            &Layout {
                window: [0.1, 0.2],
                ..Default::default()
            },
        )
        .unwrap();
        assert!(empty.svg().contains("No stored observations"));
        assert_eq!(empty.csv().matches(",outside_window").count(), 4);
        r.rows.truncate(1);
        let singleton = Figure::ordinary(&r, None, &Layout::default()).unwrap();
        assert_eq!(singleton.plot().supported_count, 1);
        assert_eq!(singleton.svg().matches("<circle ").count(), 1);
    }
    #[test]
    fn paired_figures_keep_four_inputs_independent_effect_arithmetic_and_interval_gaps() {
        let mut recordings = Vec::new();
        for branch in 0..4 {
            let mut r = recording();
            let a = (branch % 2) as f64;
            let b = (branch / 2) as f64;
            for (i, s) in r.rows.iter_mut().enumerate() {
                s.habitat.state.potable_water_kg = 100. + i as f64 * (3. * a - 5. * b + 7. * a * b);
                s.habitat.budget.delivered_w = s.habitat.state.potable_water_kg;
            }
            recordings.push(r);
        }
        let mut plan = Plan::linear(Parameter::VolumeM3, 250., 1000., 2, 4, 60.).unwrap();
        plan.schema_version = 2;
        plan.secondary = Some(Secondary {
            parameter: Parameter::HeatCapacityJK,
            values: [1e7, 1e8],
        });
        let mut d = Design {
            recordings,
            manifest: json!({"plan":plan,"common_state_fingerprint":"retained-test-common"}),
            analysis: Value::Null,
        };
        let layout = Layout {
            metric: "water".into(),
            effect: "difference_in_differences".into(),
            ..Default::default()
        };
        let f = Figure::paired(&d, &layout).unwrap();
        for (i, p) in f.series[0].points.iter().enumerate() {
            assert_eq!(p.value, Some(7. * i as f64));
        }
        assert_eq!(f.exact_samples.len(), 4);
        assert!(f.configuration_assumptions.is_some());
        assert_eq!(
            f.analysis.as_ref().unwrap()["common_state_fingerprint"],
            "retained-test-common"
        );
        d.recordings[1].rows[2].habitat.budget.seconds = 30.;
        let gap = Figure::paired(
            &d,
            &Layout {
                metric: "delivered_power".into(),
                ..layout
            },
        )
        .unwrap();
        assert!(gap.series[0].points[0].value.is_none());
        assert!(gap.series[0].points[2].value.is_none());
        assert_eq!(gap.plot().supported_count, 2);
        assert!(!gap.svg().contains("stroke-width=\"1.7\""));
    }
    #[test]
    fn complete_protected_exports_and_preflight_refusals_preserve_exact_inputs() {
        let r = recording();
        let f = Figure::ordinary(&r, None, &Layout::default()).unwrap();
        let root =
            std::env::temp_dir().join(format!("cosmogon-figure-export-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let m = f.export(&root).unwrap();
        assert_eq!(m["complete"], true);
        let bytes = std::fs::read(root.join("figure.json")).unwrap();
        let read: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            read["exact_samples"][0],
            serde_json::to_value(&r.rows).unwrap()
        );
        assert_eq!(read["source_summaries"][0], r.summary);
        assert_eq!(Layout::read(&root.join("layout.json")).unwrap(), f.layout);
        assert_eq!(
            std::fs::read_to_string(root.join("figure.svg")).unwrap(),
            f.svg()
        );
        assert!(f.export(&root).is_err());
        assert_eq!(std::fs::read(root.join("figure.json")).unwrap(), bytes);
        let mut invalid = f;
        invalid.layout.window = [0.5, 0.5];
        assert!(invalid.export(&root.join("invalid")).is_err());
        assert!(!root.join("invalid").exists());
        let empty = Recording {
            rows: vec![],
            summary: Value::Null,
        };
        assert!(Figure::ordinary(&empty, None, &Layout::default()).is_err());
        assert!(Figure::ordinary(
            &r,
            None,
            &Layout {
                metric: "invented".into(),
                ..Default::default()
            }
        )
        .is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
