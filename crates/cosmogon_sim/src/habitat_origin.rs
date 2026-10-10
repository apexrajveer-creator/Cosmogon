//! Pinned original starting worlds for new habitat experiments. No live-world rewind.
use crate::{habitat::Habitat, research, save, Universe};
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Clone)]
pub struct Origin {
    pub directory: PathBuf,
    pub habitat_id: u64,
    pub habitat: Habitat,
    pub fingerprint: String,
    pub data_versions: Value,
    world: Arc<Universe>,
}
impl Origin {
    /// The caller supplies a manifest from a fully checked completed design.
    pub fn from_manifest(directory: &Path, manifest: &Value) -> Result<Self, String> {
        if manifest["format"] != "cosmogon-habitat-sweep"
            || manifest["complete"] != true
            || !matches!(manifest["schema_version"].as_u64(), Some(1 | 2))
        {
            return Err("Original starting worlds require a completed habitat design".into());
        }
        let habitat_id = manifest["habitat_id"]
            .as_u64()
            .ok_or("Missing original habitat identity")?;
        let world = save::from_json(&crate::habitat_recording::read(
            &directory.join("base.cosmo"),
            32 * 1024 * 1024,
        )?)
        .map_err(|e| e.to_string())?;
        let fingerprint = research::state_fingerprint(&world)?;
        let data_versions =
            serde_json::to_value(save::data_versions(&world)).map_err(|e| e.to_string())?;
        if manifest["common_state_fingerprint"] != fingerprint
            || manifest["common_data_versions"] != data_versions
        {
            return Err("Original starting world/source differs from the retained design".into());
        }
        let habitat = world
            .habitats
            .iter()
            .find(|h| h.id == habitat_id)
            .ok_or("Original starting world has no recorded habitat")?
            .clone();
        Ok(Self {
            directory: directory.to_owned(),
            habitat_id,
            habitat,
            fingerprint,
            data_versions,
            world: Arc::new(world),
        })
    }
    pub fn read(directory: &Path) -> Result<Self, String> {
        let design = crate::habitat_design::read(directory)?;
        Self::from_manifest(directory, design.manifest())
    }
    pub fn cached_world(&self) -> &Universe {
        &self.world
    }
    /// Recheck request-time disk evidence against the retained pins before writing results.
    pub fn load(&self) -> Result<Universe, String> {
        let world = save::from_json(&crate::habitat_recording::read(
            &self.directory.join("base.cosmo"),
            32 * 1024 * 1024,
        )?)
        .map_err(|e| e.to_string())?;
        if research::state_fingerprint(&world)? != self.fingerprint
            || serde_json::to_value(save::data_versions(&world)).map_err(|e| e.to_string())?
                != self.data_versions
            || !world.habitats.iter().any(|h| h.id == self.habitat_id)
        {
            return Err("Saved common starting world changed after it was opened; reopen and review the source before rerunning".into());
        }
        Ok(world)
    }
    pub fn metadata(&self) -> Value {
        json!({"kind":"original_saved_common_world","directory":self.directory,"habitat_id":self.habitat_id,
            "simulation_time_s":self.world.time,"completed_habitat_time_s":self.habitat.last_time,
            "state_fingerprint":self.fingerprint,"data_versions":self.data_versions,"initial_habitat":self.habitat,
            "scope":"Retained original common starting world; typed new ordinary edits preserve finite initial stores and edit-time remainders. Original files are rechecked against these pins before a run. This does not rewind the live world or establish empirical accuracy."})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        habitat_sweep::{self, Parameter, Plan},
        sandbox::Edit,
    };
    #[test]
    fn retained_origin_reproduces_original_world_and_refuses_changed_material_before_output() {
        let mut world = Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let mut definition =
            crate::crop_process::definition_at(crate::BodyRef { system: 0, body: 3 }, 0., 0.);
        definition.config.crew = 0;
        world
            .apply_edit(Edit::CreateHabitat {
                definition: Box::new(definition),
            })
            .unwrap();
        world.advance_to(world.time + 86410.25, None, None);
        let root =
            std::env::temp_dir().join(format!("cosmogon-frozen-origin-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let plan = Plan::linear(Parameter::FirmPowerW, 0., 10000., 3, 3, 3600.).unwrap();
        habitat_sweep::execute(&world, 0, &plan, &root.join("original")).unwrap();
        let origin = Origin::read(&root.join("original")).unwrap();
        assert_eq!(origin.habitat, world.habitats[0]);
        assert_eq!(origin.cached_world().time, world.time);
        assert_eq!(
            origin.fingerprint,
            research::state_fingerprint(&world).unwrap()
        );
        let mut live = world.clone();
        live.advance_to(live.time + 86400., None, None);
        assert_ne!(
            research::state_fingerprint(&live).unwrap(),
            origin.fingerprint
        );
        assert_eq!(origin.load().unwrap().time, world.time);
        habitat_sweep::execute(
            &origin.load().unwrap(),
            origin.habitat_id,
            &plan,
            &root.join("repeated"),
        )
        .unwrap();
        for index in 0..3 {
            for file in [
                "samples.json",
                "samples.csv",
                "crop.csv",
                "report.json",
                "manifest.json",
            ] {
                assert_eq!(
                    std::fs::read(
                        root.join("original")
                            .join(format!("run-{index:03}"))
                            .join(file)
                    )
                    .unwrap(),
                    std::fs::read(
                        root.join("repeated")
                            .join(format!("run-{index:03}"))
                            .join(file)
                    )
                    .unwrap()
                );
            }
        }
        let mut altered = world.clone();
        altered.habitats[0].state.potable_water_kg -= 1.;
        altered.habitats[0].state.wastewater_kg += 1.;
        std::fs::write(
            root.join("original/base.cosmo"),
            save::to_json(&altered, "changed source").unwrap(),
        )
        .unwrap();
        assert!(origin.load().is_err());
        assert_eq!(origin.cached_world().time, world.time);
        assert_eq!(origin.habitat, world.habitats[0]);
        std::fs::remove_dir_all(root).unwrap();
    }
}
