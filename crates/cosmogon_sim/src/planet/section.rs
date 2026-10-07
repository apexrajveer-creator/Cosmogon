//! Bounded sections of the shared overview height field; no local geology or route-safety model.
use super::{landing, mola, terrain::{dir_from_lat_lon, lat_lon_from_dir}};
use cosmogon_core::{dmath::DMath, math::Vec3d};
use serde::{Deserialize, Serialize};
use std::{io::{Read, Write}, path::Path};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Point {
    pub latitude_deg: f64,
    pub longitude_east_deg: f64,
    pub landing_id: Option<String>,
}
impl Point {
    pub fn site(id: &str) -> Result<Self, String> {
        let s=landing::find(id).ok_or_else(||format!("Unknown landing reference: {id}"))?;
        Ok(Self {latitude_deg:s.latitude_north_deg,longitude_east_deg:s.longitude_east_deg,landing_id:Some(id.into())})
    }
    fn valid(&self) -> Result<(), String> {
        mola::validate_coordinates(self.latitude_deg,self.longitude_east_deg)?;
        if self.landing_id.as_deref().is_some_and(|id| landing::find(id).is_none_or(|s|s.latitude_north_deg != self.latitude_deg || s.longitude_east_deg != self.longitude_east_deg.rem_euclid(360.0))) {
            return Err("Landing identity does not match section endpoint coordinates".into());
        }
        Ok(())
    }
    fn dir(&self) -> Vec3d {let d=dir_from_lat_lon(self.latitude_deg.to_radians(),self.longitude_east_deg.to_radians());Vec3d::new(d[0],d[1],d[2])}
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub schema_version: u32,
    pub terrain_dataset: String,
    pub start: Point,
    pub end: Point,
    /// Chosen reference sphere, not an ellipsoidal/areoid geodesic.
    pub reference_radius_m: f64,
    pub samples: usize,
}
impl Default for Definition {
    fn default() -> Self {Self {schema_version:1,terrain_dataset:mola::DATASET.into(),start:Point::site("curiosity").unwrap(),end:Point::site("insight").unwrap(),reference_radius_m:3_389_500.0,samples:64}}
}
#[derive(Debug, Serialize)]
pub struct Row {
    pub index: usize,
    pub distance_m: f64,
    pub reading: mola::Reading,
    pub depends_on_provider_interpolation: bool,
}
#[derive(Debug, Serialize)]
pub struct Section {
    pub format: &'static str,
    pub schema_version: u32,
    pub definition: Definition,
    pub total_distance_m: f64,
    pub sample_spacing_m: f64,
    /// Extrema of the computed rows only, not continuous-path extrema.
    pub sampled_minimum_height_m: f64,
    pub sampled_maximum_height_m: f64,
    pub rows: Vec<Row>,
    pub limitations: &'static str,
}
impl Definition {
    pub fn read(path: &Path) -> Result<Self, String> {
        let mut data=Vec::new();std::fs::File::open(path).map_err(|e|e.to_string())?.take(65_537).read_to_end(&mut data).map_err(|e|e.to_string())?;
        if data.len()>65_536 {return Err("Section definition exceeds 64 KiB".into());}
        let definition:Self=serde_json::from_slice(&data).map_err(|e|e.to_string())?;
        definition.validate()?;Ok(definition)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {return Err("Unsupported section definition or terrain version".into());}
        mola::Dataset::from_id(&self.terrain_dataset)?;
        if !(2..=1024).contains(&self.samples) {return Err("Choose 2–1024 section samples".into());}
        if !self.reference_radius_m.is_finite() || !(1.0..=1e9).contains(&self.reference_radius_m) {return Err("Reference sphere radius must be finite and between 1 and 1000000000 metres".into());}
        self.start.valid()?;self.end.valid()?;Ok(())
    }
    pub fn calculate(&self) -> Result<Section, String> {
        self.validate()?;
        let dataset=mola::Dataset::from_id(&self.terrain_dataset)?;
        let (a,b)=(self.start.dir(),self.end.dir());
        let angle=a.cross(b).length().datan2(a.dot(b));
        if std::f64::consts::PI-angle<1e-9 {return Err("Antipodal endpoints have no unique shorter great-circle arc; choose another endpoint".into());}
        if angle<1e-10 && (self.start.latitude_deg != self.end.latitude_deg || self.start.longitude_east_deg.rem_euclid(360.0) != self.end.longitude_east_deg.rem_euclid(360.0)) {
            return Err("Endpoints coincide geometrically but use different coordinates; use identical coordinates for a zero-length section".into());
        }
        let distance=angle*self.reference_radius_m;
        let mut rows=Vec::with_capacity(self.samples);
        for index in 0..self.samples {
            let s=index as f64/(self.samples-1) as f64;
            let (lat,lon)=if index==0 {(self.start.latitude_deg,self.start.longitude_east_deg)}
                else if index==self.samples-1 {(self.end.latitude_deg,self.end.longitude_east_deg)}
                else if angle<1e-10 {(self.start.latitude_deg,self.start.longitude_east_deg)}
                else {let d=(a*((1.0-s)*angle).dsin()+b*(s*angle).dsin())/angle.dsin();let (la,lo)=lat_lon_from_dir([d.x,d.y,d.z]);(la.to_degrees(),lo.to_degrees())};
            let reading=dataset.read(lat,lon)?;
            let depends=reading.source_cells.iter().any(|c|c.weight>0.0 && c.observations==0);
            rows.push(Row {index,distance_m:s*distance,reading,depends_on_provider_interpolation:depends});
        }
        let minimum=rows.iter().map(|r|r.reading.height_above_areoid_m).fold(f64::INFINITY,f64::min);
        let maximum=rows.iter().map(|r|r.reading.height_above_areoid_m).fold(f64::NEG_INFINITY,f64::max);
        Ok(Section {format:"cosmogon-terrain-section",schema_version:1,definition:self.clone(),total_distance_m:distance,sample_spacing_m:distance/(self.samples-1) as f64,sampled_minimum_height_m:minimum,sampled_maximum_height_m:maximum,rows,
            limitations:"A section of the pinned MOLA overview (grid resolution is in the source metadata). Uniform distances are on the chosen sphere, not an ellipsoidal or areoid geodesic. Finer sample spacing does not add source resolution. Heights are bilinear areoid-referenced values; zero-shot cells are provider-interpolated. Unknown height/position uncertainty remains unavailable. Sample extrema are not continuous-path extrema. No local slopes, rover-route or landing-safety assessment."})
    }
}
impl Section {
    /// Exclusive directory creation protects existing exports, including empty directories.
    /// An I/O failure leaves a clearly incomplete directory for inspection, never reports success.
    pub fn export(&self, path: &Path) -> Result<(), String> {
        let definition=serde_json::to_vec_pretty(&self.definition).map_err(|e|e.to_string())?;
        let section=serde_json::to_vec_pretty(self).map_err(|e|e.to_string())?;
        let sources=serde_json::to_vec_pretty(&serde_json::json!({"format":"cosmogon-terrain-section-sources","schema_version":1,"mola":serde_json::from_str::<serde_json::Value>(mola::Dataset::from_id(&self.definition.terrain_dataset)?.metadata()).map_err(|e|e.to_string())?,"landing_references":landing::registry(),"coordinate_registration":landing::REGISTRATION})).map_err(|e|e.to_string())?;
        let mut csv=String::from("index,distance_m,latitude_north_deg,longitude_east_deg,height_above_areoid_m,height_uncertainty_m,provider_interpolated_support");
        for i in 0..4 {csv.push_str(&format!(",cell{i}_row,cell{i}_column,cell{i}_latitude_north_deg,cell{i}_longitude_east_deg,cell{i}_median_height_m,cell{i}_observations,cell{i}_weight"));}
        csv.push('\n');
        for row in &self.rows {
            let r=&row.reading;
            csv.push_str(&format!("{},{},{},{},{},,{}",row.index,row.distance_m,r.latitude_deg,r.longitude_east_deg,r.height_above_areoid_m,row.depends_on_provider_interpolation));
            for c in &r.source_cells {csv.push_str(&format!(",{},{},{},{},{},{},{}",c.row,c.column,c.latitude_deg,c.longitude_east_deg,c.median_height_m,c.observations,c.weight));}
            csv.push('\n');
        }
        if let Some(parent)=path.parent().filter(|p|!p.as_os_str().is_empty()) {std::fs::create_dir_all(parent).map_err(|e|e.to_string())?;}
        std::fs::create_dir(path).map_err(|e|format!("Cannot create a new section export (existing folders are protected): {e}"))?;
        for (name,data) in [("definition.json",definition),("section.json",section),("sources.json",sources),("samples.csv",csv.into_bytes())] {
            let mut file=std::fs::OpenOptions::new().write(true).create_new(true).open(path.join(name)).map_err(|e|format!("Incomplete section export at {}: {e}",path.display()))?;
            file.write_all(&data).map_err(|e|format!("Incomplete section export at {}: {e}",path.display()))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn point(lat:f64,lon:f64)->Point {Point {latitude_deg:lat,longitude_east_deg:lon,landing_id:None}}
    #[test]
    fn finer_sections_pin_source_and_export_matching_metadata() {
        let d=Definition {terrain_dataset:mola::DATASET_16.into(),samples:8,..Default::default()};let s=d.calculate().unwrap();
        for row in &s.rows {assert_eq!(row.reading.dataset,mola::DATASET_16);assert_eq!(row.reading.height_above_areoid_m,mola::Dataset::Sixteen.read(row.reading.latitude_deg,row.reading.longitude_east_deg).unwrap().height_above_areoid_m);}
        let path=std::env::temp_dir().join(format!("cosmogon-section16-{}",std::process::id()));let _=std::fs::remove_dir_all(&path);s.export(&path).unwrap();
        let replay=Definition::read(&path.join("definition.json")).unwrap().calculate().unwrap();assert_eq!(serde_json::to_value(s).unwrap(),serde_json::to_value(replay).unwrap());
        let sources:serde_json::Value=serde_json::from_slice(&std::fs::read(path.join("sources.json")).unwrap()).unwrap();assert_eq!(sources["mola"]["id"],"mgs-mola-megdr-16ppd");assert_eq!(sources["mola"]["cell_degrees"],0.0625);std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn known_arcs_endpoints_seam_and_poles_use_shared_heights() {
        for (start,end,angle) in [(point(0.0,0.0),point(0.0,90.0),std::f64::consts::FRAC_PI_2),(point(0.0,359.0),point(0.0,1.0),2f64.to_radians()),(point(90.0,0.0),point(0.0,0.0),std::f64::consts::FRAC_PI_2)] {
            let d=Definition {start,end,samples:11,..Default::default()};let section=d.calculate().unwrap();
            assert!((section.total_distance_m-angle*d.reference_radius_m).abs()<1e-7);
            assert_eq!(section.rows[0].reading.latitude_deg,d.start.latitude_deg);
            assert_eq!(section.rows.last().unwrap().reading.longitude_east_deg,d.end.longitude_east_deg);
            for (i,row) in section.rows.iter().enumerate() {
                assert_eq!(row.index,i);assert!((row.distance_m-section.sample_spacing_m*i as f64).abs()<1e-7);
                let dir=dir_from_lat_lon(row.reading.latitude_deg.to_radians(),row.reading.longitude_east_deg.to_radians());
                assert!((mola::elevation_m(dir)-row.reading.height_above_areoid_m).abs()<1e-7);
            }
            let repeat=d.calculate().unwrap();assert_eq!(serde_json::to_string(&section).unwrap(),serde_json::to_string(&repeat).unwrap());
        }
    }
    #[test]
    fn ambiguous_invalid_and_coincident_sections_are_explicit() {
        let d=Definition {start:point(0.0,0.0),end:point(0.0,180.0),..Default::default()};assert!(d.calculate().unwrap_err().contains("Antipodal"));
        let d=Definition {start:point(0.0,0.0),end:point(0.0,0.0),..Default::default()};let section=d.calculate().unwrap();assert_eq!(section.total_distance_m,0.0);assert_eq!(section.sampled_minimum_height_m,section.sampled_maximum_height_m);
        for samples in [0,1,1025,usize::MAX] {assert!(Definition {samples,..Default::default()}.calculate().is_err());}
        assert!(Definition {reference_radius_m:f64::NAN,..Default::default()}.calculate().is_err());
        assert!(Definition {terrain_dataset:"future".into(),..Default::default()}.calculate().is_err());
        assert!(Definition {start:Point {latitude_deg:0.0,..Point::site("curiosity").unwrap()},..Default::default()}.calculate().is_err());
        assert!(Definition {start:point(90.0,0.0),end:point(90.0,90.0),..Default::default()}.calculate().is_err());
    }
    #[test]
    fn export_replays_and_protects_existing_files_and_unknown_definitions() {
        let dir=std::env::temp_dir().join(format!("cosmogon-section-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let section=Definition::default().calculate().unwrap();section.export(&dir).unwrap();
        let definition=Definition::read(&dir.join("definition.json")).unwrap();let replay=definition.calculate().unwrap();
        assert_eq!(serde_json::to_string(&section).unwrap(),serde_json::to_string(&replay).unwrap());
        let before=std::fs::read(dir.join("samples.csv")).unwrap();assert!(section.export(&dir).is_err());assert_eq!(before,std::fs::read(dir.join("samples.csv")).unwrap());
        let csv=std::str::from_utf8(&before).unwrap();assert_eq!(csv.lines().count(),section.rows.len()+1);assert!(csv.lines().all(|l|l.split(',').count()==35));
        let mut v=serde_json::to_value(&definition).unwrap();v["schema_version"]=serde_json::json!(999);std::fs::write(dir.join("future.json"),serde_json::to_vec(&v).unwrap()).unwrap();assert!(Definition::read(&dir.join("future.json")).is_err());
        std::fs::write(dir.join("oversized.json"),vec![b' ';65_537]).unwrap();assert!(Definition::read(&dir.join("oversized.json")).unwrap_err().contains("64 KiB"));
        std::fs::create_dir(dir.join("existing-empty")).unwrap();assert!(section.export(&dir.join("existing-empty")).is_err());assert_eq!(std::fs::read_dir(dir.join("existing-empty")).unwrap().count(),0);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
