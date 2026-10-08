//! Native input customization. Camera and HUD consume one validated mapping.
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down,
    RollLeft,
    RollRight,
    LookLeft,
    LookRight,
    LookUp,
    LookDown,
    ToggleFlight,
    OrbitSelection,
    ViewSystem,
    Pause,
    SpeedUp,
    SpeedUpAlt,
    SpeedDown,
    SpeedDownAlt,
    HideInterface,
    Photo,
    Help,
    Debug,
}
impl Action {
    pub const ALL: [Self; 24] = [
        Self::Forward,
        Self::Backward,
        Self::Left,
        Self::Right,
        Self::Up,
        Self::Down,
        Self::RollLeft,
        Self::RollRight,
        Self::LookLeft,
        Self::LookRight,
        Self::LookUp,
        Self::LookDown,
        Self::ToggleFlight,
        Self::OrbitSelection,
        Self::ViewSystem,
        Self::Pause,
        Self::SpeedUp,
        Self::SpeedUpAlt,
        Self::SpeedDown,
        Self::SpeedDownAlt,
        Self::HideInterface,
        Self::Photo,
        Self::Help,
        Self::Debug,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Forward => "Forward / orbit zoom in",
            Self::Backward => "Backward / orbit zoom out",
            Self::Left => "Move left / orbit left",
            Self::Right => "Move right / orbit right",
            Self::Up => "Move up",
            Self::Down => "Move down",
            Self::RollLeft => "Roll left",
            Self::RollRight => "Roll right",
            Self::LookLeft => "Look left",
            Self::LookRight => "Look right",
            Self::LookUp => "Look up",
            Self::LookDown => "Look down",
            Self::ToggleFlight => "Switch orbit / free flight",
            Self::OrbitSelection => "Orbit selection",
            Self::ViewSystem => "View star system",
            Self::Pause => "Pause / resume simulation",
            Self::SpeedUp => "Increase simulation speed",
            Self::SpeedUpAlt => "Increase speed (alternate)",
            Self::SpeedDown => "Decrease simulation speed",
            Self::SpeedDownAlt => "Decrease speed (alternate)",
            Self::HideInterface => "Hide / show interface",
            Self::Photo => "Take clean photo",
            Self::Help => "Show / hide help",
            Self::Debug => "Show / hide developer overlay",
        }
    }
    pub fn default_code(self) -> &'static str {
        match self {
            Self::Forward => "KeyW",
            Self::Backward => "KeyS",
            Self::Left => "KeyA",
            Self::Right => "KeyD",
            Self::Up => "KeyR",
            Self::Down => "KeyC",
            Self::RollLeft => "KeyQ",
            Self::RollRight => "KeyE",
            Self::LookLeft => "ArrowLeft",
            Self::LookRight => "ArrowRight",
            Self::LookUp => "ArrowUp",
            Self::LookDown => "ArrowDown",
            Self::ToggleFlight => "KeyG",
            Self::OrbitSelection => "KeyF",
            Self::ViewSystem => "Home",
            Self::Pause => "Space",
            Self::SpeedUp => "Period",
            Self::SpeedUpAlt => "Equal",
            Self::SpeedDown => "Comma",
            Self::SpeedDownAlt => "Minus",
            Self::HideInterface => "Tab",
            Self::Photo => "KeyP",
            Self::Help => "F1",
            Self::Debug => "F3",
        }
    }
    pub fn camera(self) -> bool {
        Self::ALL[..15].contains(&self)
    }
}
// An explicit portable subset; no native unidentified codes or modifier/chord assignments.
pub const CODES: &[(&str, KeyCode)] = &[
    ("KeyA", KeyCode::KeyA),
    ("KeyB", KeyCode::KeyB),
    ("KeyC", KeyCode::KeyC),
    ("KeyD", KeyCode::KeyD),
    ("KeyE", KeyCode::KeyE),
    ("KeyF", KeyCode::KeyF),
    ("KeyG", KeyCode::KeyG),
    ("KeyH", KeyCode::KeyH),
    ("KeyI", KeyCode::KeyI),
    ("KeyJ", KeyCode::KeyJ),
    ("KeyK", KeyCode::KeyK),
    ("KeyL", KeyCode::KeyL),
    ("KeyM", KeyCode::KeyM),
    ("KeyN", KeyCode::KeyN),
    ("KeyO", KeyCode::KeyO),
    ("KeyP", KeyCode::KeyP),
    ("KeyQ", KeyCode::KeyQ),
    ("KeyR", KeyCode::KeyR),
    ("KeyS", KeyCode::KeyS),
    ("KeyT", KeyCode::KeyT),
    ("KeyU", KeyCode::KeyU),
    ("KeyV", KeyCode::KeyV),
    ("KeyW", KeyCode::KeyW),
    ("KeyX", KeyCode::KeyX),
    ("KeyY", KeyCode::KeyY),
    ("KeyZ", KeyCode::KeyZ),
    ("Digit0", KeyCode::Digit0),
    ("Digit1", KeyCode::Digit1),
    ("Digit2", KeyCode::Digit2),
    ("Digit3", KeyCode::Digit3),
    ("Digit4", KeyCode::Digit4),
    ("Digit5", KeyCode::Digit5),
    ("Digit6", KeyCode::Digit6),
    ("Digit7", KeyCode::Digit7),
    ("Digit8", KeyCode::Digit8),
    ("Digit9", KeyCode::Digit9),
    ("ArrowLeft", KeyCode::ArrowLeft),
    ("ArrowRight", KeyCode::ArrowRight),
    ("ArrowUp", KeyCode::ArrowUp),
    ("ArrowDown", KeyCode::ArrowDown),
    ("Home", KeyCode::Home),
    ("End", KeyCode::End),
    ("PageUp", KeyCode::PageUp),
    ("PageDown", KeyCode::PageDown),
    ("Space", KeyCode::Space),
    ("Tab", KeyCode::Tab),
    ("Enter", KeyCode::Enter),
    ("Backspace", KeyCode::Backspace),
    ("Comma", KeyCode::Comma),
    ("Period", KeyCode::Period),
    ("Equal", KeyCode::Equal),
    ("Minus", KeyCode::Minus),
    ("F1", KeyCode::F1),
    ("F2", KeyCode::F2),
    ("F3", KeyCode::F3),
    ("F4", KeyCode::F4),
    ("F5", KeyCode::F5),
    ("F6", KeyCode::F6),
    ("F7", KeyCode::F7),
    ("F8", KeyCode::F8),
    ("F9", KeyCode::F9),
    ("F10", KeyCode::F10),
    ("F11", KeyCode::F11),
    ("F12", KeyCode::F12),
];
fn code(value: &str) -> Option<KeyCode> {
    CODES.iter().find(|(id, _)| *id == value).map(|(_, k)| *k)
}
pub fn key_label(id: &str) -> String {
    match id {
        "ArrowLeft" => "←",
        "ArrowRight" => "→",
        "ArrowUp" => "↑",
        "ArrowDown" => "↓",
        "Comma" => ",",
        "Period" => ".",
        "Equal" => "=",
        "Minus" => "-",
        _ => id
            .strip_prefix("Key")
            .or_else(|| id.strip_prefix("Digit"))
            .unwrap_or(id),
    }
    .into()
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub format: String,
    pub schema_version: u32,
    #[serde(deserialize_with = "read_keys")]
    pub keys: BTreeMap<Action, String>,
}
fn read_keys<'de, D: serde::Deserializer<'de>>(d: D) -> Result<BTreeMap<Action, String>, D::Error> {
    struct Unique;
    impl<'de> serde::de::Visitor<'de> for Unique {
        type Value = BTreeMap<Action, String>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("one unique physical key per action")
        }
        fn visit_map<M: serde::de::MapAccess<'de>>(
            self,
            mut map: M,
        ) -> Result<Self::Value, M::Error> {
            let mut result = BTreeMap::new();
            while let Some((action, key)) = map.next_entry::<Action, String>()? {
                if result.insert(action, key).is_some() {
                    return Err(serde::de::Error::custom(format!(
                        "Duplicate action {}",
                        action.label()
                    )));
                }
            }
            Ok(result)
        }
    }
    d.deserialize_map(Unique)
}
impl Default for Definition {
    fn default() -> Self {
        Self {
            format: "cosmogon-input-bindings".into(),
            schema_version: 1,
            keys: Action::ALL
                .into_iter()
                .map(|a| (a, a.default_code().into()))
                .collect(),
        }
    }
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.format != "cosmogon-input-bindings" || self.schema_version != 1 {
            return Err("Unsupported input-binding format/version".into());
        }
        if self.keys.len() != Action::ALL.len()
            || Action::ALL.iter().any(|a| !self.keys.contains_key(a))
        {
            return Err("Every supported action needs one explicit key".into());
        }
        let mut used = BTreeSet::new();
        for (a, key) in &self.keys {
            if code(key).is_none() {
                return Err(format!(
                    "{}: unsupported physical key {key}; Escape and modifiers are reserved",
                    a.label()
                ));
            }
            if !used.insert(key) {
                return Err(format!(
                    "Physical key {} is assigned to more than one action",
                    key_label(key)
                ));
            }
        }
        Ok(())
    }
    fn bytes(path: &Path) -> std::io::Result<Vec<u8>> {
        let m = std::fs::metadata(path)?;
        if !m.is_file() || m.len() > 16 * 1024 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Bindings require a regular file no larger than 16 KiB",
            ));
        }
        let f = std::fs::File::open(path)?;
        let mut bytes = Vec::new();
        f.take(16 * 1024 + 1).read_to_end(&mut bytes)?;
        if bytes.len() > 16 * 1024 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Bindings exceed 16 KiB",
            ));
        }
        Ok(bytes)
    }
    fn parse(bytes: &[u8]) -> Result<Self, String> {
        let value: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        value.validate()?;
        Ok(value)
    }
    pub fn read(path: &Path) -> Result<Self, String> {
        Self::parse(&Self::bytes(path).map_err(|e| e.to_string())?)
    }
    pub fn export(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        if let Some(p) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
        }
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| format!("Existing binding files are protected: {e}"))?;
        f.write_all(&bytes)
            .map_err(|e| format!("Incomplete export: {e}"))
    }
}
#[derive(Resource)]
pub struct Bindings {
    pub definition: Definition,
    pub capturing: Option<Action>,
    pub blocked: bool,
    pub writable: bool,
    pub dirty: bool,
    pub status: Option<String>,
    pub pane: usize,
    pub selected: Action,
    pub filter: String,
    pub import_path: String,
    pub export_path: String,
    path: PathBuf,
    loaded_bytes: Option<Vec<u8>>,
}
impl Default for Bindings {
    fn default() -> Self {
        Self::load(crate::persistence::data_dir().join("input-bindings.json"))
    }
}
impl Bindings {
    pub fn load(path: PathBuf) -> Self {
        let (definition, writable, status, loaded_bytes) = match Definition::bytes(&path) {
            Ok(bytes) => match Definition::parse(&bytes) {
                Ok(d) => (d, true, None, Some(bytes)),
                Err(e) => (
                    Definition::default(),
                    false,
                    Some(format!(
                        "Saved controls are protected; using runtime defaults: {e}"
                    )),
                    None,
                ),
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                (Definition::default(), true, None, None)
            }
            Err(e) => (
                Definition::default(),
                false,
                Some(format!(
                    "Saved controls are protected; using runtime defaults: {e}"
                )),
                None,
            ),
        };
        Self {
            loaded_bytes,
            definition,
            capturing: None,
            blocked: false,
            writable,
            dirty: false,
            status,
            pane: 0,
            selected: Action::Forward,
            filter: String::new(),
            import_path: String::new(),
            export_path: String::new(),
            path,
        }
    }
    pub fn reload(&mut self) {
        let path = self.path.clone();
        *self = Self::load(path);
        let message = self
            .status
            .clone()
            .unwrap_or_else(|| "Saved controls reloaded.".into());
        self.cancel(&format!("{message} Release keys to resume."));
    }
    pub fn key(&self, a: Action) -> KeyCode {
        code(
            self.definition
                .keys
                .get(&a)
                .map(String::as_str)
                .unwrap_or(a.default_code()),
        )
        .expect("Validated binding key")
    }
    pub fn label(&self, a: Action) -> String {
        key_label(
            self.definition
                .keys
                .get(&a)
                .map(String::as_str)
                .unwrap_or(a.default_code()),
        )
    }
    pub fn held(&self, keys: &ButtonInput<KeyCode>, a: Action) -> bool {
        !self.blocked && keys.pressed(self.key(a))
    }
    pub fn pressed(&self, keys: &ButtonInput<KeyCode>, a: Action) -> bool {
        !self.blocked && keys.just_pressed(self.key(a))
    }
    pub fn camera_input(&self, keys: &ButtonInput<KeyCode>) -> bool {
        Action::ALL
            .into_iter()
            .filter(|a| a.camera())
            .any(|a| self.held(keys, a))
    }
    pub fn axis(&self, keys: &ButtonInput<KeyCode>, positive: Action, negative: Action) -> f64 {
        (self.held(keys, positive) as i32 - self.held(keys, negative) as i32) as f64
    }
    pub fn assign(&mut self, a: Action, key: &str) -> Result<(), String> {
        if let Some((other, _)) = self
            .definition
            .keys
            .iter()
            .find(|(other, value)| **other != a && value.as_str() == key)
        {
            return Err(format!(
                "{} already controls {}. Choose an unused key or reassign that action first.",
                key_label(key),
                other.label()
            ));
        }
        let mut next = self.definition.clone();
        next.keys.insert(a, key.into());
        next.validate()?;
        self.definition = next;
        self.dirty = true;
        self.status = Some(format!(
            "{} → {} · changes apply now; save explicitly to keep them",
            a.label(),
            key_label(key)
        ));
        Ok(())
    }
    pub fn begin_capture(&mut self, a: Action) {
        self.capturing = Some(a);
        self.blocked = true;
        self.status = Some(format!(
            "Press a single supported physical key for {}. Escape cancels.",
            a.label()
        ));
    }
    pub fn cancel(&mut self, reason: &str) {
        self.capturing = None;
        self.blocked = true;
        self.status = Some(reason.into());
    }
    pub fn poll_keys(&mut self, keys: &ButtonInput<KeyCode>) {
        let Some(action) = self.capturing else {
            if keys.get_pressed().next().is_none() {
                self.blocked = false;
            }
            return;
        };
        self.blocked = true;
        if keys.just_pressed(KeyCode::Escape) {
            self.cancel("Key assignment cancelled. Release keys to resume controls.");
            return;
        }
        if [
            KeyCode::ControlLeft,
            KeyCode::ControlRight,
            KeyCode::SuperLeft,
            KeyCode::SuperRight,
            KeyCode::ShiftLeft,
            KeyCode::ShiftRight,
            KeyCode::AltLeft,
            KeyCode::AltRight,
        ]
        .into_iter()
        .any(|k| keys.pressed(k))
        {
            self.status = Some("Release modifiers and press a single key. Escape cancels.".into());
            return;
        }
        let pressed: Vec<_> = keys.get_just_pressed().copied().collect();
        if pressed.is_empty() {
            return;
        }
        if pressed.len() != 1 {
            self.status = Some("Press only one key at a time. Escape cancels.".into());
            return;
        }
        let Some((id, _)) = CODES.iter().find(|(_, k)| *k == pressed[0]) else {
            self.status=Some("That key is reserved or unsupported. Choose a letter, digit, navigation or F1–F12 key.".into());
            return;
        };
        match self.assign(action, id) {
            Ok(()) => {
                self.capturing = None;
            }
            Err(e) => self.status = Some(e),
        }
    }
    pub fn import(&mut self, path: &Path) -> Result<(), String> {
        let d = Definition::read(path)?;
        self.definition = d;
        self.dirty = true;
        self.cancel("Bindings opened for this run; release keys and save explicitly to keep them.");
        Ok(())
    }
    pub fn reset(&mut self) {
        self.definition = Definition::default();
        self.dirty = true;
        self.cancel("Default controls restored for this run. Save explicitly to keep them.");
    }
    pub fn persist(&mut self) -> Result<(), String> {
        if !self.writable {
            return Err("Unsupported or invalid saved controls remain protected. Export to a new file instead.".into());
        }
        match (&self.loaded_bytes, Definition::bytes(&self.path)) {
            (Some(previous), Ok(current)) if *previous == current => {}
            (None, Err(e)) if e.kind() == std::io::ErrorKind::NotFound => {}
            _ => return Err(
                "Saved controls changed outside this editor; reload them or export to a new file."
                    .into(),
            ),
        }
        self.definition.validate()?;
        let bytes = serde_json::to_vec_pretty(&self.definition).map_err(|e| e.to_string())?;
        let p = self.path.parent().ok_or("Invalid controls folder")?;
        std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
        let tmp = self
            .path
            .with_extension(format!("{}.tmp", std::process::id()));
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
            .map_err(|e| e.to_string())?;
        let written = file.write_all(&bytes).and_then(|_| file.sync_all());
        drop(file);
        let result = written.and_then(|_| std::fs::rename(&tmp, &self.path));
        if let Err(e) = result {
            let _ = std::fs::remove_file(&tmp);
            return Err(e.to_string());
        }
        self.loaded_bytes = Some(bytes);
        self.dirty = false;
        self.status = Some("Controls saved for later launches.".into());
        Ok(())
    }
    pub fn metadata(&self) -> serde_json::Value {
        serde_json::json!({"definition":self.definition,"capturing":self.capturing,"input_suspended_until_release":self.blocked,"unsaved_changes":self.dirty,"saved_file_writable":self.writable,"scope":"Single physical keys; Escape and command/control chords are fixed. Mouse/controller/chord rebinding is not implemented."})
    }
}
pub fn command_modifier(keys: &ButtonInput<KeyCode>) -> bool {
    [
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]
    .into_iter()
    .any(|k| keys.pressed(k))
}

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Bindings>()
            .add_systems(Startup, launch)
            .add_systems(
                Update,
                poll.before(crate::state::Frame::Camera)
                    .run_if(in_state(crate::state::AppState::Observing)),
            )
            .add_systems(OnExit(crate::state::AppState::Observing), leave);
    }
}
fn launch(
    mut b: ResMut<Bindings>,
    args: Res<crate::args::Args>,
    mut exit: MessageWriter<bevy::app::AppExit>,
) {
    b.pane = match args.panel.as_deref() {
        Some("controls-everyday") => 1,
        Some("controls-files") => 2,
        _ => 0,
    };
    if let Some(path) = &args.input_bindings {
        if let Err(e) = b.import(Path::new(path)) {
            error!("Cannot open controls: {e}");
            b.status = Some(format!("Cannot open controls: {e}"));
            exit.write(bevy::app::AppExit::error());
        }
    }
}
fn poll(mut b: ResMut<Bindings>, keys: Res<ButtonInput<KeyCode>>, ui: Res<crate::ui::UiState>) {
    if b.capturing.is_some()
        && (!ui.camera_open || ui.camera_tab != 4 || ui.hidden || ui.photo_mode || ui.pause_menu)
    {
        b.cancel("Controls editor closed; assignment cancelled. Release keys to resume.");
    }
    b.poll_keys(&keys);
}
fn leave(mut b: ResMut<Bindings>) {
    if b.capturing.is_some() {
        b.cancel("World closed; key assignment cancelled.");
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn dir() -> PathBuf {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let p = std::env::temp_dir().join(format!(
            "cosmogon-bindings-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&p).unwrap();
        p
    }
    #[test]
    fn defaults_preserve_actions_and_rebinding_drives_held_and_edge_input() {
        let p = dir();
        let mut b = Bindings::load(p.join("controls.json"));
        b.definition.validate().unwrap();
        assert_eq!(b.key(Action::Forward), KeyCode::KeyW);
        assert_eq!(b.key(Action::Pause), KeyCode::Space);
        b.assign(Action::Forward, "KeyI").unwrap();
        let mut keys = ButtonInput::default();
        keys.press(KeyCode::KeyW);
        assert!(!b.held(&keys, Action::Forward));
        keys.press(KeyCode::KeyI);
        assert!(b.pressed(&keys, Action::Forward));
        assert!(b.camera_input(&keys));
        keys.clear();
        assert!(!b.pressed(&keys, Action::Forward));
        assert!(b.held(&keys, Action::Forward));
        let local =
            bevy::math::DVec3::new(0.0, 0.0, b.axis(&keys, Action::Backward, Action::Forward));
        let mut f =
            crate::navigation::Flight::new(bevy::math::DVec3::ZERO, bevy::math::DQuat::IDENTITY);
        f.advance(local * 10.0, 0.1, false, 4.0);
        assert_eq!(f.position.z, -1.0);
        std::fs::remove_dir_all(p).unwrap();
    }
    #[test]
    fn capture_conflicts_cancellation_modifiers_and_release_are_explicit() {
        let p = dir();
        let mut b = Bindings::load(p.join("controls.json"));
        let before = b.definition.clone();
        let mut keys = ButtonInput::default();
        b.begin_capture(Action::Forward);
        keys.press(KeyCode::KeyS);
        b.poll_keys(&keys);
        assert_eq!(b.definition, before);
        assert_eq!(b.capturing, Some(Action::Forward));
        assert!(!b.held(&keys, Action::Backward));
        keys.reset_all();
        keys.press(KeyCode::ShiftLeft);
        keys.press(KeyCode::KeyI);
        b.poll_keys(&keys);
        assert_eq!(b.definition, before);
        keys.reset_all();
        keys.press(KeyCode::KeyI);
        b.poll_keys(&keys);
        assert_eq!(b.key(Action::Forward), KeyCode::KeyI);
        assert!(b.blocked);
        assert!(!b.pressed(&keys, Action::Forward));
        keys.reset_all();
        b.poll_keys(&keys);
        assert!(!b.blocked);
        b.begin_capture(Action::Photo);
        keys.press(KeyCode::Escape);
        b.poll_keys(&keys);
        assert!(b.capturing.is_none() && b.blocked);
        assert_eq!(b.key(Action::Photo), KeyCode::KeyP);
        keys.reset_all();
        b.poll_keys(&keys);
        assert!(!b.blocked);
        assert!(b.assign(Action::Photo, "Escape").is_err());
        assert!(b.assign(Action::Photo, "SuperLeft").is_err());
        std::fs::remove_dir_all(p).unwrap();
    }
    #[test]
    fn strict_schema_rejects_future_duplicate_missing_and_unknown_actions() {
        let d = Definition::default();
        let mut v = serde_json::to_value(&d).unwrap();
        v["schema_version"] = 2.into();
        assert!(Definition::parse(&serde_json::to_vec(&v).unwrap()).is_err());
        v = serde_json::to_value(&d).unwrap();
        v["extra"] = true.into();
        assert!(Definition::parse(&serde_json::to_vec(&v).unwrap()).is_err());
        v = serde_json::to_value(&d).unwrap();
        v["keys"].as_object_mut().unwrap().remove("forward");
        assert!(Definition::parse(&serde_json::to_vec(&v).unwrap()).is_err());
        v = serde_json::to_value(&d).unwrap();
        v["keys"]["invented_action"] = "KeyI".into();
        assert!(Definition::parse(&serde_json::to_vec(&v).unwrap()).is_err());
        let text = serde_json::to_string(&d).unwrap().replace(
            "\"forward\":\"KeyW\"",
            "\"forward\":\"KeyW\",\"forward\":\"KeyI\"",
        );
        assert!(Definition::parse(text.as_bytes()).is_err());
    }
    #[test]
    fn protected_exports_and_invalid_import_preserve_runtime_configuration() {
        let p = dir();
        let file = p.join("export.json");
        Definition::default().export(&file).unwrap();
        let bytes = std::fs::read(&file).unwrap();
        assert!(Definition::default().export(&file).is_err());
        assert_eq!(std::fs::read(&file).unwrap(), bytes);
        let mut b = Bindings::load(p.join("primary.json"));
        b.assign(Action::Forward, "KeyI").unwrap();
        let before = b.definition.clone();
        std::fs::write(p.join("invalid.json"), b"{}").unwrap();
        assert!(b.import(&p.join("invalid.json")).is_err());
        assert_eq!(b.definition, before);
        std::fs::write(p.join("large.json"), vec![b' '; 16385]).unwrap();
        assert!(Definition::read(&p.join("large.json")).is_err());
        assert!(Definition::read(&p).is_err());
        std::fs::remove_dir_all(p).unwrap();
    }
    #[test]
    fn settings_persistence_reopens_and_refuses_external_or_future_overwrites() {
        let p = dir();
        let path = p.join("primary.json");
        let mut b = Bindings::load(path.clone());
        b.assign(Action::Forward, "KeyI").unwrap();
        b.persist().unwrap();
        assert!(!b.dirty);
        let loaded = Bindings::load(path.clone());
        assert_eq!(loaded.definition, b.definition);
        std::fs::write(&path, b"{\"schema_version\":99}").unwrap();
        assert!(b.persist().is_err());
        let protected = std::fs::read(&path).unwrap();
        let mut future = Bindings::load(path.clone());
        assert!(!future.writable);
        future.reset();
        assert!(future.persist().is_err());
        assert_eq!(std::fs::read(&path).unwrap(), protected);
        std::fs::remove_dir_all(p).unwrap();
    }
    #[test]
    fn closing_editor_cancels_capture_before_actions_resume() {
        let p = dir();
        let mut b = Bindings::load(p.join("primary.json"));
        b.begin_capture(Action::Pause);
        let mut app = App::new();
        app.insert_resource(b)
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<crate::ui::UiState>()
            .add_systems(Update, poll);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyI);
        app.update();
        let b = app.world().resource::<Bindings>();
        assert!(b.capturing.is_none() && b.blocked);
        assert_eq!(b.key(Action::Pause), KeyCode::Space);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.update();
        assert!(!app.world().resource::<Bindings>().blocked);
        std::fs::remove_dir_all(p).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn fifo_is_refused_without_opening_the_pipe() {
        let p = dir();
        let pipe = p.join("pipe");
        assert!(std::process::Command::new("mkfifo")
            .arg(&pipe)
            .status()
            .unwrap()
            .success());
        assert!(Definition::read(&pipe).is_err());
        let b = Bindings::load(pipe);
        assert!(!b.writable);
        std::fs::remove_dir_all(p).unwrap();
    }
}
