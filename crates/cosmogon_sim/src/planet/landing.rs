//! Historical landing references, not current rover positions. No Universe state or network I/O.
use serde::Serialize;

pub const VERSION: &str = "mars-landing-references@2026-10-07.1";
pub const ACCESSED: &str = "2026-10-07";
pub const REGISTRATION: &str = "Source north latitude/east longitude interpreted in the MOLA planetocentric body-fixed frame for overview navigation. These summaries do not declare a complete mapping datum/frame or positional covariance; metre-level registration is unverified.";
const GISS: &str = "https://www.giss.nasa.gov/tools/mars24/help/landers.html";

#[derive(Debug, Serialize)]
pub struct Landing {
    pub id: &'static str,
    pub mission: &'static str,
    pub body: &'static str,
    pub landing_utc_date: &'static str,
    pub latitude_north_deg: f64,
    pub longitude_east_deg: f64,
    pub source_url: &'static str,
    /// Date of the cited source narrative/record, not the mission's present status.
    pub source_date: &'static str,
    pub positional_uncertainty_m: Option<f64>,
}

pub static SITES: [Landing; 4] = [
    Landing { id: "perseverance", mission: "Perseverance", body: "Mars", landing_utc_date: "2021-02-18", latitude_north_deg: 18.44463, longitude_east_deg: 77.45088, source_url: "https://ntrs.nasa.gov/citations/20210024644", source_date: "2021-11-19", positional_uncertainty_m: None },
    Landing { id: "curiosity", mission: "Curiosity", body: "Mars", landing_utc_date: "2012-08-06", latitude_north_deg: -4.59, longitude_east_deg: 137.44, source_url: GISS, source_date: "2023-12-09", positional_uncertainty_m: None },
    Landing { id: "insight", mission: "InSight", body: "Mars", landing_utc_date: "2018-11-26", latitude_north_deg: 4.50, longitude_east_deg: 135.62, source_url: GISS, source_date: "2023-12-09", positional_uncertainty_m: None },
    Landing { id: "viking-1", mission: "Viking 1", body: "Mars", landing_utc_date: "1976-07-20", latitude_north_deg: 22.27, longitude_east_deg: 312.05, source_url: GISS, source_date: "2023-12-09", positional_uncertainty_m: None },
];

pub fn find(id: &str) -> Option<&'static Landing> { SITES.iter().find(|s| s.id == id) }

pub fn registry() -> serde_json::Value {
    serde_json::json!({"format":"cosmogon-landing-references", "schema_version":1,
        "version":VERSION,"sources_accessed":ACCESSED,"sites":SITES,
        "coordinate_registration":REGISTRATION,
        "limitations":"Historical touchdown reference points only; no live mission status, rover paths, imagery or measured uncertainty. Decimal precision is not positional accuracy."})
}

/// A portable reading with source support. The caller checks the selected body's terrain.
pub fn probe(latitude: f64, longitude: f64, site_id: Option<&str>) -> Result<serde_json::Value, String> {
    probe_dataset(super::mola::Dataset::Four, latitude, longitude, site_id)
}
pub fn probe_dataset(dataset: super::mola::Dataset, latitude: f64, longitude: f64, site_id: Option<&str>) -> Result<serde_json::Value, String> {
    let site = site_id.map(|id| find(id).ok_or_else(|| format!("Unknown landing reference: {id}"))).transpose()?;
    if site.is_some_and(|s| s.latitude_north_deg != latitude || s.longitude_east_deg != longitude.rem_euclid(360.0)) {
        return Err("Landing identity does not match the probe coordinates".into());
    }
    let reading = dataset.read(latitude, longitude)?;
    let height = dataset.elevation_m(super::terrain::dir_from_lat_lon(latitude.to_radians(), longitude.to_radians()));
    Ok(serde_json::json!({"format":"cosmogon-surface-probe","schema_version":2,
        "terrain_dataset":dataset.id(),"terrain_metadata":serde_json::from_str::<serde_json::Value>(dataset.metadata()).map_err(|e|e.to_string())?,"terrain_source":super::mola::SOURCE_URL,
        "landing_registry":VERSION,"sources_accessed":ACCESSED,"landing_reference":site,
        "coordinate_registration":REGISTRATION,"reading":reading,"authoritative_height_m":height,
        "frame":"IAU2000 planetocentric latitude; east longitude; GMM3 degree/order 50 areoid height",
        "sampling":format!("Bilinear between {}-degree grid pixel centres; zero-observation cells are provider-interpolated",1.0/dataset.ppd() as f64),
        "limitations":"Global overview (source spacing recorded in metadata), not landing safety/local relief. No per-cell uncertainty or propagated covariance. Rendering uses a spherical base radius; areoid shape is not modelled. Colours, climate and subpixel shading are generated/modelled separately."}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn references_are_bounded_unique_and_traceable() {
        for (i, s) in SITES.iter().enumerate() {
            assert!(SITES[..i].iter().all(|other| other.id != s.id));
            assert!((-90.0..=90.0).contains(&s.latitude_north_deg));
            assert!((0.0..360.0).contains(&s.longitude_east_deg));
            assert!(s.source_url.starts_with("https://"));
            assert!(s.positional_uncertainty_m.is_none());
            let record = probe(s.latitude_north_deg, s.longitude_east_deg, Some(s.id)).unwrap();
            assert_eq!(record["landing_reference"]["id"], s.id);
            assert!((record["authoritative_height_m"].as_f64().unwrap() - record["reading"]["height_above_areoid_m"].as_f64().unwrap()).abs() < 1e-7);
        }
        assert_eq!(find("perseverance").unwrap().latitude_north_deg, 18.44463);
        assert_eq!(find("curiosity").unwrap().latitude_north_deg, -4.59);
        assert!(probe(0.0, 0.0, Some("perseverance")).is_err());
        assert!(probe(0.0, 0.0, Some("missing")).is_err());
        assert!(probe(f64::NAN, 0.0, None).is_err());
    }
}
