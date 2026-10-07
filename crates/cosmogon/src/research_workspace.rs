//! Presentation choices belong to the user, independently of saved universe state.
use bevy::prelude::*;
use cosmogon_sim::research::{ParameterId, PARAMETERS};
use serde::{Deserialize, Serialize};
use std::{io::Read, path::PathBuf};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Guidance {
    Explore,
    #[default]
    Learn,
    Research,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeUnit {
    #[default]
    Seconds,
    Days,
    Years,
}
impl TimeUnit {
    pub fn factor(self) -> f64 {
        match self {
            Self::Seconds => 1.0,
            Self::Days => 86400.0,
            Self::Years => 365.25 * 86400.0,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Seconds => "s",
            Self::Days => "days",
            Self::Years => "Julian years",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scale {
    #[default]
    Linear,
    Log,
    Fractional,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlotChoice {
    pub parameter: ParameterId,
    pub scale: Scale,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Choices {
    pub guidance: Guidance,
    pub favorites: Vec<ParameterId>,
    pub favorite_only: bool,
    pub plots: Vec<PlotChoice>,
    pub time_unit: TimeUnit,
    pub comparison_scale: Scale,
    pub comparison_difference: bool,
    /// Fractions of the current recording's duration, never a claim of shared absolute epochs.
    pub range: Option<(f64, f64)>,
}
impl Default for Choices {
    fn default() -> Self {
        Self {
            guidance: Guidance::Learn,
            favorites: vec![
                ParameterId::Mass,
                ParameterId::Radius,
                ParameterId::Eccentricity,
            ],
            favorite_only: false,
            plots: vec![
                ParameterId::Temperature,
                ParameterId::Gravity,
                ParameterId::Insolation,
                ParameterId::Habitability,
            ]
            .into_iter()
            .map(|parameter| PlotChoice {
                parameter,
                scale: Scale::Linear,
            })
            .collect(),
            time_unit: TimeUnit::Seconds,
            comparison_scale: Scale::Linear,
            comparison_difference: false,
            range: None,
        }
    }
}
impl Choices {
    fn validate(&self) -> Result<(), String> {
        if self.favorites.len() > PARAMETERS.len() || self.plots.is_empty() || self.plots.len() > 4
        {
            return Err(
                "Workspace permits up to four graphs and one favorite per parameter".into(),
            );
        }
        if self
            .favorites
            .iter()
            .enumerate()
            .any(|(i, p)| self.favorites[..i].contains(p))
            || self
                .plots
                .iter()
                .enumerate()
                .any(|(i, p)| self.plots[..i].iter().any(|v| v.parameter == p.parameter))
        {
            return Err("Workspace choices contain duplicate parameters".into());
        }
        if self.range.is_some_and(|(a, b)| {
            !a.is_finite() || !b.is_finite() || a < 0.0 || b > 1.0 || b - a < 1e-6
        }) {
            return Err("Saved graph range is invalid".into());
        }
        Ok(())
    }
}
#[derive(Serialize, Deserialize)]
struct File {
    format: String,
    schema_version: u32,
    choices: Choices,
}
#[derive(Resource)]
pub struct ResearchWorkspace {
    pub choices: Choices,
    pub dirty: bool,
    pub status: Option<String>,
    pub cursor: Option<f64>,
    writable: bool,
    path: PathBuf,
}
impl Default for ResearchWorkspace {
    fn default() -> Self {
        Self::load(crate::persistence::data_dir().join("research-workspace.json"))
    }
}
impl ResearchWorkspace {
    pub fn load(path: PathBuf) -> Self {
        let mut out = Self {
            choices: Choices::default(),
            dirty: false,
            status: None,
            cursor: None,
            writable: true,
            path,
        };
        let result = (|| -> Result<Option<Choices>, String> {
            let file = match std::fs::File::open(&out.path) {
                Ok(f) => f,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(e) => return Err(e.to_string()),
            };
            let mut bytes = Vec::new();
            file.take(65537)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            if bytes.len() > 65536 {
                return Err("Workspace file exceeds 64 KiB".into());
            }
            let file: File = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            if file.format != "cosmogon-research-workspace" || file.schema_version != 1 {
                return Err("Unsupported workspace format/version".into());
            }
            file.choices.validate()?;
            Ok(Some(file.choices))
        })();
        match result {
            Ok(Some(c)) => out.choices = c,
            Ok(None) => {}
            Err(e) => {
                out.writable = false;
                out.status = Some(format!("Saved workspace preserved; writes disabled: {e}"));
            }
        }
        out
    }
    pub fn favorite(&mut self, p: ParameterId) {
        if let Some(i) = self.choices.favorites.iter().position(|v| *v == p) {
            self.choices.favorites.remove(i);
        } else {
            self.choices.favorites.push(p);
        }
        self.dirty = true;
    }
    pub fn save(&mut self) -> Result<(), String> {
        if !self.writable {
            return Err(
                "Existing invalid/newer workspace is preserved; writes are disabled".into(),
            );
        }
        self.write(&self.path)?;
        self.dirty = false;
        self.status = Some("Workspace choices saved. Simulation unchanged.".into());
        Ok(())
    }
    fn write(&self, path: &std::path::Path) -> Result<(), String> {
        self.choices.validate()?;
        let bytes = serde_json::to_vec_pretty(&File {
            format: "cosmogon-research-workspace".into(),
            schema_version: 1,
            choices: self.choices.clone(),
        })
        .map_err(|e| e.to_string())?;
        let parent = path.parent().ok_or("Workspace path has no parent")?;
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let temp = path.with_extension(format!("{}.tmp", std::process::id()));
        std::fs::write(&temp, bytes).map_err(|e| e.to_string())?;
        std::fs::rename(&temp, path).map_err(|e| e.to_string())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn choices_round_trip_and_unknown_versions_remain_protected() {
        let path = std::env::temp_dir().join(format!(
            "cosmogon-workspace-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut w = ResearchWorkspace::load(path.clone());
        w.favorite(ParameterId::Albedo);
        w.choices.range = Some((0.25, 0.75));
        w.choices.time_unit = TimeUnit::Days;
        w.save().unwrap();
        let loaded = ResearchWorkspace::load(path.clone());
        assert!(loaded.choices.favorites.contains(&ParameterId::Albedo));
        assert_eq!(loaded.choices.range, Some((0.25, 0.75)));
        assert_eq!(loaded.choices.time_unit, TimeUnit::Days);
        let bytes =
            b"{\"format\":\"cosmogon-research-workspace\",\"schema_version\":99,\"choices\":{}}";
        std::fs::write(&path, bytes).unwrap();
        let mut unknown = ResearchWorkspace::load(path.clone());
        assert!(unknown.save().is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        std::fs::remove_file(path).unwrap();
        let invalid = Choices {
            plots: vec![],
            ..Default::default()
        };
        assert!(invalid.validate().is_err());
    }
}
