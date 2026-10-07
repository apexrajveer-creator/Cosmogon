//! NASA PDS MOLA overview topography and observation coverage. No generated relief.
use super::terrain::lat_lon_from_dir;
use serde::Serialize;
use std::sync::OnceLock;

pub const WIDTH: usize = 1440;
pub const HEIGHT: usize = 720;
pub const DATASET: &str = "mgs-mola-megdr-4ppd@2003-grid-pds4-label-1.1";
pub const SOURCE_URL: &str =
    "https://pds-geosciences.wustl.edu/mgs/urn-nasa-pds-mgs_mola_topography_derived/";
pub const DATUM: &str = "GMM3 (mgm1025) degree/order 50 areoid";
pub const METADATA: &str = include_str!("../../data/mola/metadata.json");
static ELEVATION: &[u8] = include_bytes!("../../data/mola/mars_mola_elevation.bin");
static COUNTS: &[u8] = include_bytes!("../../data/mola/mars_mola_counts.bin");
fn grids() -> &'static (Vec<i16>, Vec<u16>) {
    static GRID: OnceLock<(Vec<i16>, Vec<u16>)> = OnceLock::new();
    GRID.get_or_init(|| {
        (
            ELEVATION
                .chunks_exact(2)
                .map(|b| i16::from_le_bytes([b[0], b[1]]))
                .collect(),
            COUNTS
                .chunks_exact(2)
                .map(|b| u16::from_le_bytes([b[0], b[1]]))
                .collect(),
        )
    })
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
fn sample_grid(lat: f64, lon: f64) -> Reading {
    let (elevation, counts) = grids();
    let x = lon.rem_euclid(360.0) * 4.0 - 0.5;
    let y = ((90.0 - lat) * 4.0 - 0.5).clamp(0.0, (HEIGHT - 1) as f64);
    let (x0, y0) = (x.floor() as i64, y.floor() as usize);
    let (fx, fy) = (x - x.floor(), y - y.floor());
    let cell = |dx: i64, dy: usize, weight: f64| {
        let column = (x0 + dx).rem_euclid(WIDTH as i64) as usize;
        let row = (y0 + dy).min(HEIGHT - 1);
        let index = row * WIDTH + column;
        Cell {
            row,
            column,
            latitude_deg: 89.875 - row as f64 * 0.25,
            longitude_east_deg: 0.125 + column as f64 * 0.25,
            median_height_m: elevation[index],
            observations: counts[index],
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
        dataset: DATASET,
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
    if !latitude_deg.is_finite()
        || !(-90.0..=90.0).contains(&latitude_deg)
        || !longitude_deg.is_finite()
        || !(-360.0..=360.0).contains(&longitude_deg)
    {
        return Err("Latitude must be -90..90°, longitude -360..360°, both finite".into());
    }
    Ok(sample_grid(latitude_deg, longitude_deg))
}
/// Rendering and physical surface models use this same bilinear height in metres.
pub fn elevation_m(direction: [f64; 3]) -> f64 {
    let (lat, lon) = lat_lon_from_dir(direction);
    sample_grid(lat.to_degrees(), lon.to_degrees()).height_above_areoid_m
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planet::terrain::dir_from_lat_lon;

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
        let (e, c) = grids();
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
