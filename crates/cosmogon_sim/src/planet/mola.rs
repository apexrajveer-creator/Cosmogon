//! NASA PDS MOLA overview topography and observation coverage. No generated relief.
use super::terrain::lat_lon_from_dir;
use serde::Serialize;


pub const WIDTH: usize = 1440;
pub const HEIGHT: usize = 720;
pub const DATASET: &str = "mgs-mola-megdr-4ppd@2003-grid-pds4-label-1.1";
pub const SOURCE_URL: &str =
    "https://pds-geosciences.wustl.edu/mgs/urn-nasa-pds-mgs_mola_topography_derived/";
pub const DATUM: &str = "GMM3 (mgm1025) degree/order 50 areoid";
pub const METADATA: &str = include_str!("../../data/mola/metadata.json");
static ELEVATION: &[u8] = include_bytes!("../../data/mola/mars_mola_elevation.bin");
static COUNTS: &[u8] = include_bytes!("../../data/mola/mars_mola_counts.bin");
pub const DATASET_16: &str = "mgs-mola-megdr-16ppd@2003-grid-pds4-label-1.1";
/// Both immutable grids ship with the app. Reads decode just the contributing cells;
/// no second heap copy, download, mutable cache or silent procedural fallback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dataset { Four, Sixteen }
impl Dataset {
    pub fn from_id(id: &str) -> Result<Self, String> {
        match id { DATASET => Ok(Self::Four), DATASET_16 => Ok(Self::Sixteen), _ => Err(format!("Unsupported MOLA dataset: {id}")) }
    }
    pub fn id(self) -> &'static str {match self {Self::Four => DATASET, Self::Sixteen => DATASET_16}}
    pub fn ppd(self) -> usize {match self {Self::Four => 4, Self::Sixteen => 16}}
    pub fn metadata(self) -> &'static str {match self {Self::Four => METADATA, Self::Sixteen => include_str!("../../data/mola16/metadata.json")}}
    pub fn label(self) -> &'static str {match self {Self::Four => "MOLA 4ppd · 0.25° / 14.82 km", Self::Sixteen => "MOLA 16ppd · 0.0625° / 3.70 km"}}
    fn bytes(self) -> (&'static [u8], &'static [u8]) {match self {
        Self::Four => (ELEVATION, COUNTS),
        Self::Sixteen => (include_bytes!("../../data/mola16/mars_mola_elevation.bin"), include_bytes!("../../data/mola16/mars_mola_counts.bin")),
    }}
    pub fn read(self, latitude_deg: f64, longitude_deg: f64) -> Result<Reading, String> {
        validate_coordinates(latitude_deg, longitude_deg)?;
        Ok(sample_grid(self, latitude_deg, longitude_deg))
    }
    pub fn elevation_m(self, direction: [f64; 3]) -> f64 {
        let (lat, lon) = lat_lon_from_dir(direction);
        sample_grid(self, lat.to_degrees(), lon.to_degrees()).height_above_areoid_m
    }
}
pub fn registry() -> serde_json::Value {
    serde_json::json!({"format":"cosmogon-terrain-datasets", "schema_version":1, "availability":"Embedded immutable data; no network/cache required", "datasets":[
        serde_json::from_str::<serde_json::Value>(Dataset::Four.metadata()).expect("embedded metadata"),
        serde_json::from_str::<serde_json::Value>(Dataset::Sixteen.metadata()).expect("embedded metadata")
    ]})
}
#[derive(Clone, Debug, Serialize)]
pub struct Cell {
    pub row: usize,
    pub column: usize,
    pub latitude_deg: f64,
    pub longitude_east_deg: f64,
    pub median_height_m: i16,
    pub observations: u16,
    pub weight: f64,
}
#[derive(Clone, Debug, Serialize)]
pub struct Reading {
    pub dataset: &'static str,
    pub latitude_deg: f64,
    pub longitude_east_deg: f64,
    pub height_above_areoid_m: f64,
    pub source_cells: [Cell; 4],
    pub uncertainty_m: Option<f64>,
}
fn sample_grid(dataset: Dataset, lat: f64, lon: f64) -> Reading {
    let (elevation, counts) = dataset.bytes();
    let ppd = dataset.ppd() as f64;
    let (width, height) = (360 * dataset.ppd(), 180 * dataset.ppd());
    let x = lon.rem_euclid(360.0) * ppd - 0.5;
    let y = ((90.0 - lat) * ppd - 0.5).clamp(0.0, (height - 1) as f64);
    let (x0, y0) = (x.floor() as i64, y.floor() as usize);
    let (fx, fy) = (x - x.floor(), y - y.floor());
    let cell = |dx: i64, dy: usize, weight: f64| {
        let column = (x0 + dx).rem_euclid(width as i64) as usize;
        let row = (y0 + dy).min(height - 1);
        let index = row * width + column;
        Cell {
            row,
            column,
            latitude_deg: 90.0 - (0.5 + row as f64) / ppd,
            longitude_east_deg: (0.5 + column as f64) / ppd,
            median_height_m: i16::from_le_bytes([elevation[index * 2], elevation[index * 2 + 1]]),
            observations: u16::from_le_bytes([counts[index * 2], counts[index * 2 + 1]]),
            weight,
        }
    };
    let source_cells = [
        cell(0, 0, (1.0 - fx) * (1.0 - fy)),
        cell(1, 0, fx * (1.0 - fy)),
        cell(0, 1, (1.0 - fx) * fy),
        cell(1, 1, fx * fy),
    ];
    Reading {
        dataset: dataset.id(),
        latitude_deg: lat,
        longitude_east_deg: lon.rem_euclid(360.0),
        height_above_areoid_m: source_cells
            .iter()
            .map(|c| c.median_height_m as f64 * c.weight)
            .sum(),
        source_cells,
        uncertainty_m: None,
    }
}
/// User coordinates: planetocentric latitude, east-positive longitude in degrees.
pub fn read(latitude_deg: f64, longitude_deg: f64) -> Result<Reading, String> {
    Dataset::Four.read(latitude_deg, longitude_deg)
}
pub fn validate_coordinates(latitude_deg: f64, longitude_deg: f64) -> Result<(), String> {
    if !latitude_deg.is_finite() || !(-90.0..=90.0).contains(&latitude_deg)
        || !longitude_deg.is_finite() || !(-360.0..=360.0).contains(&longitude_deg) {
        return Err("Latitude must be -90..90°, longitude -360..360°, both finite".into());
    }
    Ok(())
}
/// Compatibility entry point for the original pinned 4ppd field.
pub fn elevation_m(direction: [f64; 3]) -> f64 {Dataset::Four.elevation_m(direction)}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planet::terrain::dir_from_lat_lon;

    #[test]
    fn explicit_sources_are_deterministic_saved_and_preserve_orbital_state() {
        use crate::{save, Scenario, Universe, UniverseSettings, BodyRef, sandbox::Edit};
        use super::super::terrain::{ElevationData,Terrain};
        let mut a=Universe::new(UniverseSettings {scenario:Scenario::SolarSystemLab,..Default::default()});
        let r=BodyRef {system:0,body:a.systems[0].find_body("Mars").unwrap() as u32};
        let mut b=a.clone();
        let dynamics=serde_json::to_value(&a.systems[0].dynamics).unwrap();let t=a.time;
        let old=Terrain::of(a.body(r)).cache_key();
        let edit=Edit::SetMarsTerrain {body:r,dataset:ElevationData::MarsMola16};
        a.apply_edit(edit.clone()).unwrap();b.apply_edit(edit).unwrap();
        assert_eq!(serde_json::to_value(&a).unwrap(),serde_json::to_value(&b).unwrap());
        assert_eq!(a.time,t);assert_eq!(dynamics,serde_json::to_value(&a.systems[0].dynamics).unwrap());
        assert_ne!(old,Terrain::of(a.body(r)).cache_key());
        assert!(!save::data_versions(&a).contains_key("nasa-pds-mgs-mola-megdr"));
        assert_eq!(save::data_versions(&a)["nasa-pds-mgs-mola-megdr-16ppd"],DATASET_16);
        let json=save::to_json(&a,"Sixteen").unwrap();let loaded=save::from_json(&json).unwrap();assert_eq!(loaded.body(r).elevation_data,Some(ElevationData::MarsMola16));
        let mut bad:serde_json::Value=serde_json::from_str(&json).unwrap();bad["header"]["data_versions"]["nasa-pds-mgs-mola-megdr-16ppd"]=serde_json::json!("future");assert!(save::from_json(&bad.to_string()).is_err());
        bad["header"]["data_versions"].as_object_mut().unwrap().remove("nasa-pds-mgs-mola-megdr-16ppd");assert!(save::from_json(&bad.to_string()).is_err());
        let before=serde_json::to_string(&a).unwrap();
        assert!(a.apply_edit(Edit::SetMarsTerrain {body:r,dataset:ElevationData::Earth}).is_err());
        let earth=BodyRef {system:0,body:a.systems[0].find_body("Earth").unwrap() as u32};
        assert!(a.apply_edit(Edit::SetMarsTerrain {body:earth,dataset:ElevationData::MarsMola16}).is_err());
        assert_eq!(before,serde_json::to_string(&a).unwrap());
    }
    #[test]
    fn sixteen_pixel_support_poles_seam_and_terrain_agree() {
        use super::super::terrain::{Terrain,ElevationData,METRES_PER_UNIT};
        let dataset=Dataset::Sixteen;
        let (e,c)=dataset.bytes();assert_eq!(e.len(),5760*2880*2);assert_eq!(e.len(),c.len());
        assert_eq!(c.chunks_exact(2).filter(|b|b[0]==0 && b[1]==0).count(),903558);
        let heights=e.chunks_exact(2).map(|b|i16::from_le_bytes([b[0],b[1]]));
        assert_eq!(heights.clone().min(),Some(-8177));assert_eq!(heights.max(),Some(21171));
        // Fixtures read independently from the provider's signed big-endian payloads.
        for (lat,lon,height,shots) in [(89.96875,0.03125,-1965,0),(-89.96875,359.96875,3814,0),(18.46875,77.46875,-2568,49),(-4.59375,137.46875,-4509,24),(18.59375,226.15625,20072,42),(-42.59375,70.15625,-6474,0)] {
            let reading=dataset.read(lat,lon).unwrap();assert_eq!(reading.height_above_areoid_m,height as f64);assert_eq!(reading.source_cells[0].observations,shots);assert_eq!(reading.source_cells[0].weight,1.0);
        }
        for (lat,lon) in [(89.96875,0.03125),(-89.96875,359.96875),(18.44463,77.45088),(-4.59,137.44),(-90.0,0.0),(90.0,270.0),(0.0,-360.0)] {
            let r=dataset.read(lat,lon).unwrap();assert_eq!(r.dataset,DATASET_16);assert!(r.uncertainty_m.is_none());
            assert!((r.source_cells.iter().map(|c|c.weight).sum::<f64>()-1.0).abs()<1e-12);
            for cell in &r.source_cells {assert!(cell.row<2880 && cell.column<5760);assert_eq!(cell.latitude_deg,90.0-(cell.row as f64+0.5)/16.0);assert_eq!(cell.longitude_east_deg,(cell.column as f64+0.5)/16.0);}
            let d=dir_from_lat_lon(lat.to_radians(),lon.to_radians());let terrain=Terrain {seed:4,data:Some(ElevationData::MarsMola16)};
            assert!((terrain.elevation(d)*METRES_PER_UNIT-r.height_above_areoid_m).abs()<1e-7);
        }
        assert_eq!(dataset.read(0.0,0.0).unwrap().height_above_areoid_m,dataset.read(0.0,360.0).unwrap().height_above_areoid_m);
        assert!(dataset.read(91.0,0.0).is_err());assert!(dataset.read(0.0,f64::NAN).is_err());assert!(Dataset::from_id("future").is_err());
        assert_ne!(dataset.read(-4.59,137.44).unwrap().height_above_areoid_m,Dataset::Four.read(-4.59,137.44).unwrap().height_above_areoid_m);
    }
    #[test]
    fn new_and_legacy_terrain_survive_save_roundtrip() {
        use crate::planet::terrain::ElevationData;
        use crate::{save, Scenario, Universe, UniverseSettings};
        let mut u = Universe::new(UniverseSettings {
            scenario: Scenario::Sol,
            seed: 1,
            ..Default::default()
        });
        let mars = u.systems[0].find_body("Mars").unwrap();
        assert_eq!(
            u.systems[0].bodies[mars].elevation_data,
            Some(ElevationData::MarsMola)
        );
        let loaded = save::from_json(&save::to_json(&u, "MOLA").unwrap()).unwrap();
        assert_eq!(
            loaded.systems[0].bodies[mars].elevation_data,
            Some(ElevationData::MarsMola)
        );
        assert_eq!(
            save::data_versions(&loaded)["nasa-pds-mgs-mola-megdr"],
            DATASET
        );
        // Loading a pre-MOLA world must preserve its original procedural surface.
        u.systems[0].bodies[mars].elevation_data = None;
        let loaded = save::from_json(&save::to_json(&u, "Legacy").unwrap()).unwrap();
        assert_eq!(loaded.systems[0].bodies[mars].elevation_data, None);
        assert!(!save::data_versions(&loaded).contains_key("nasa-pds-mgs-mola-megdr"));
        assert!(save::data_versions(&loaded).contains_key("noaa-etopo5-earth"));
    }
    #[test]
    fn pinned_grid_preserves_observations_orientation_and_wrapping() {
        let e: Vec<i16> = ELEVATION.chunks_exact(2).map(|b|i16::from_le_bytes([b[0],b[1]])).collect();
        let c: Vec<u16> = COUNTS.chunks_exact(2).map(|b|u16::from_le_bytes([b[0],b[1]])).collect();
        assert_eq!(e.len(), WIDTH * HEIGHT);
        assert_eq!(c.len(), e.len());
        assert_eq!(
            (*e.iter().min().unwrap(), *e.iter().max().unwrap()),
            (-8068, 21134)
        );
        assert_eq!(c.iter().filter(|v| **v == 0).count(), 9137);
        let r = read(18.625, 226.125).unwrap();
        assert_eq!(r.height_above_areoid_m, 20009.0);
        assert_eq!(r.source_cells[0].observations, 732);
        assert_eq!(
            read(-4.625, 137.375).unwrap().height_above_areoid_m,
            -4458.0
        );
        assert_eq!(
            read(-42.625, 70.125).unwrap().height_above_areoid_m,
            -6602.0
        );
        for lat in [-90.0, -45.0, 0.0, 45.0, 90.0] {
            assert_eq!(
                read(lat, 0.0).unwrap().height_above_areoid_m,
                read(lat, 360.0).unwrap().height_above_areoid_m
            );
            let r = read(lat, 33.43).unwrap();
            let min = r
                .source_cells
                .iter()
                .map(|c| c.median_height_m)
                .min()
                .unwrap() as f64;
            let max = r
                .source_cells
                .iter()
                .map(|c| c.median_height_m)
                .max()
                .unwrap() as f64;
            assert!((min..=max).contains(&r.height_above_areoid_m));
            assert!((r.source_cells.iter().map(|c| c.weight).sum::<f64>() - 1.0).abs() < 1e-12);
            assert!(
                (elevation_m(dir_from_lat_lon(lat.to_radians(), 33.43f64.to_radians()))
                    - r.height_above_areoid_m)
                    .abs()
                    < 1e-8
            );
        }
        assert!(read(f64::NAN, 0.0).is_err());
        assert!(read(91.0, 0.0).is_err());
    }
}
