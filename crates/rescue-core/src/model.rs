//! Domain model. Everything is plain serde data so it can round-trip through localStorage JSON.
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

/// Opaque id; a String so the web layer can generate it without a UUID dependency.
pub type AnimalId = String;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Species {
    Dog,
    Cat,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Sex {
    Male,
    Female,
    Unknown,
}

/// Tri-state for facts a rescue often can't know (altered, temperament).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum YesNoUnknown {
    Yes,
    No,
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Energy {
    Low,
    Medium,
    High,
}

/// Kanban lifecycle of an animal in the rescue.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Status {
    Intake,
    VetCheck,
    MedicalHold,
    Available,
    AdoptionPending,
    Outcome,
}

impl Status {
    /// Board column order.
    pub const ALL: [Status; 6] = [
        Status::Intake,
        Status::VetCheck,
        Status::MedicalHold,
        Status::Available,
        Status::AdoptionPending,
        Status::Outcome,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Status::Intake => "Intake",
            Status::VetCheck => "Vet Check",
            Status::MedicalHold => "Medical Hold",
            Status::Available => "Available",
            Status::AdoptionPending => "Adoption Pending",
            Status::Outcome => "Outcome",
        }
    }
}

/// How an animal left the rescue's care (recorded in the disposition record).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum OutcomeMethod {
    Adopted,
    Transferred,
    ReturnedToOwner,
    Died,
    Euthanized,
}

impl OutcomeMethod {
    pub fn label(&self) -> &'static str {
        match self {
            OutcomeMethod::Adopted => "Adopted",
            OutcomeMethod::Transferred => "Transferred",
            OutcomeMethod::ReturnedToOwner => "Returned to owner",
            OutcomeMethod::Died => "Died",
            OutcomeMethod::Euthanized => "Euthanized",
        }
    }
}

/// Who the animal came from; the required fields differ for people vs. agencies.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Source {
    Person { name: String, address: String, drivers_license: String },
    Agency { agency: String, official_title: String, address: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Microchip {
    pub number: String,
    pub manufacturer: Option<String>,
    pub location: Option<String>,
}

/// Any one identifier satisfies the "individually identified" record requirement.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Identification {
    pub tag: Option<String>,
    pub cage_card: Option<String>,
    pub tattoo: Option<String>,
    pub microchip: Option<Microchip>,
}

impl Identification {
    pub fn has_any(&self) -> bool {
        self.tag.is_some()
            || self.cage_card.is_some()
            || self.tattoo.is_some()
            || self.microchip.is_some()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum HealthKind {
    Vaccination,
    RabiesVaccination,
    Treatment,
    Medication,
    Procedure,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HealthEntry {
    pub date: NaiveDate,
    pub kind: HealthKind,
    pub description: String,
    pub reason: Option<String>,
    pub result: Option<String>,
    pub rabies_certificate: Option<String>,
}

/// Outcome record; only filled once the animal reaches `Status::Outcome`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Disposition {
    pub date: Option<NaiveDate>,
    pub method: OutcomeMethod,
    pub recipient_name: Option<String>,
    pub recipient_address: Option<String>,
    pub circumstances: Option<String>,
    pub euthanasia_by: Option<String>,
    pub euthanasia_method: Option<String>,
}

/// Adopter-facing details used for the adoption post text.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Story {
    pub breed: Option<String>,
    pub age_text: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub color_markings: Option<String>,
    pub weight_lbs: Option<f32>,
    /// Free text like "about 50 lbs grown"; older saved data lacks it.
    #[serde(default)]
    pub size_potential: Option<String>,
    pub altered: YesNoUnknown,
    pub good_with_dogs: YesNoUnknown,
    pub good_with_cats: YesNoUnknown,
    pub good_with_kids: YesNoUnknown,
    pub house_trained: YesNoUnknown,
    pub energy: Option<Energy>,
    pub personality: Option<String>,
    pub special_needs: Option<String>,
    pub adoption_fee: Option<u32>,
}

/// Times are naive: the web layer supplies local time, core never reads a clock.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Animal {
    pub id: AnimalId,
    pub name: String,
    pub species: Species,
    pub sex: Sex,
    pub received_at: NaiveDateTime,
    pub source: Option<Source>,
    pub out_of_state: bool,
    pub health_certificate: Option<String>,
    pub identification: Identification,
    pub health: Vec<HealthEntry>,
    pub story: Story,
    pub photo: Option<String>,
    pub status: Status,
    pub status_changed_at: NaiveDateTime,
    pub disposition: Option<Disposition>,
}

/// Rescue-wide branding used in post text.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub rescue_name: String,
    pub phone: String,
    pub email: String,
    pub website: String,
    pub hashtags: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            rescue_name: "Your Rescue Name".into(),
            phone: "(402) 555-0100".into(),
            email: "adopt@example.org".into(),
            website: String::new(),
            hashtags: vec!["#AdoptDontShop".into(), "#NebraskaRescue".into()],
        }
    }
}
