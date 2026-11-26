//! Persistence (localStorage) and the shared reactive app state.
use crate::clock;
use leptos::prelude::*;
use rescue_core::model::{Animal, Settings};
use rescue_core::samples::sample_animals;
use rescue_core::store_data::{StoreData, StoreError, SCHEMA_VERSION};
use std::rc::Rc;

pub const STORAGE_KEY: &str = "rescue-demo-v1";
const RESET_NOTICE: &str = "Saved demo data couldn't be read and was reset";

/// Storage seam so tests/later tasks can swap in an in-memory store.
pub trait Store {
    /// Missing key is not an error: it yields fresh sample data.
    fn load(&self) -> Result<StoreData, StoreError>;
    fn save(&self, d: &StoreData) -> Result<(), String>;
}

pub struct LocalStore;

impl LocalStore {
    fn storage() -> Result<web_sys::Storage, String> {
        window()
            .local_storage()
            .map_err(|_| "localStorage unavailable".to_string())?
            .ok_or_else(|| "localStorage unavailable".to_string())
    }
}

impl Store for LocalStore {
    fn load(&self) -> Result<StoreData, StoreError> {
        let raw = Self::storage()
            .ok()
            .and_then(|s| s.get_item(STORAGE_KEY).ok().flatten());
        match raw {
            None => Ok(fresh_samples()),
            Some(json) => StoreData::from_json(&json),
        }
    }

    fn save(&self, d: &StoreData) -> Result<(), String> {
        Self::storage()?
            .set_item(STORAGE_KEY, &d.to_json())
            .map_err(|_| "Couldn't save (browser storage full or blocked). Storage may be full \u{2014} remove a photo or use Reset demo on the Animals page.".to_string())
    }
}

/// Three demo animals with bundled (relative-URL) photos.
pub fn fresh_samples() -> StoreData {
    let photos = [
        Some("assets/samples/biscuit.svg".to_string()),
        Some("assets/samples/pepper.svg".to_string()),
        Some("assets/samples/moose.svg".to_string()),
    ];
    StoreData {
        version: SCHEMA_VERSION,
        animals: sample_animals(clock::now(), &photos),
        settings: Settings::default(),
    }
}

/// Context object shared by all pages. Copy, so closures can capture it freely.
#[derive(Clone, Copy)]
pub struct AppState {
    pub data: RwSignal<StoreData>,
    pub notice: RwSignal<Option<String>>,
    store: StoredValue<Rc<dyn Store>, LocalStorage>,
}

impl AppState {
    pub fn init(store: Box<dyn Store>) -> Self {
        let (data, notice) = match store.load() {
            Ok(d) => (d, None),
            // Corrupt or wrong-version data: fall back to samples and tell the user.
            Err(_) => (fresh_samples(), Some(RESET_NOTICE.to_string())),
        };
        AppState {
            data: RwSignal::new(data),
            notice: RwSignal::new(notice),
            store: StoredValue::new_local(Rc::from(store)),
        }
    }

    pub fn persist(&self) -> Result<(), String> {
        let store = self.store.get_value();
        self.data.with_untracked(|d| store.save(d))
    }

    /// Insert or replace by id, then save. Persists a clone first and only publishes it to
    /// the signal on success, so a failed save never leaves memory ahead of storage.
    pub fn upsert(&self, a: Animal) -> Result<(), String> {
        let mut next = self.data.get_untracked();
        match next.animals.iter_mut().find(|x| x.id == a.id) {
            Some(slot) => *slot = a,
            None => next.animals.push(a),
        }
        self.store.get_value().save(&next)?;
        self.data.set(next);
        Ok(())
    }

    /// Replace the rescue branding and save. Same persist-clone-first rule as `upsert`.
    pub fn save_settings(&self, s: Settings) -> Result<(), String> {
        let mut next = self.data.get_untracked();
        next.settings = s;
        self.store.get_value().save(&next)?;
        self.data.set(next);
        Ok(())
    }

    /// Restore the sample data (also clears any corrupt blob) and the notice.
    pub fn reset(&self) {
        self.data.set(fresh_samples());
        // Never silent: if the save fails, tell the user the reset won't survive a reload.
        match self.persist() {
            Ok(()) => self.notice.set(None),
            Err(e) => self.notice.set(Some(format!("Couldn't save the reset demo data: {e}"))),
        }
    }
}
