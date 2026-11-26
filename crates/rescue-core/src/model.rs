//! Domain model. Everything is plain serde data so it can round-trip through localStorage JSON.
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

/// Opaque id; a String so the web layer can generate it without a UUID dependency.
pub type AnimalId = String;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Species {
    Horse,
    Donkey,
    Goat,
    Sheep,
    Chicken,
    Duck,
    Goose,
    Rabbit,
    GuineaPig,
    Dog,
    Cat,
    Other,
}

/// Groups species that share record-keeping rules and form fields.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Family {
    Equine,
    SmallRuminant,
    Poultry,
    SmallMammal,
    Dog,
    Cat,
    Other,
}

impl Species {
    /// Display / select order (farm animals first, dogs and cats are occasional).
    pub const ALL: [Species; 12] = [
        Species::Horse,
        Species::Donkey,
        Species::Goat,
        Species::Sheep,
        Species::Chicken,
        Species::Duck,
        Species::Goose,
        Species::Rabbit,
        Species::GuineaPig,
        Species::Dog,
        Species::Cat,
        Species::Other,
    ];

    /// Display name (kept separate from `Debug` so UI text never depends on derive output).
    pub fn label(&self) -> &'static str {
        match self {
            Species::Horse => "Horse",
            Species::Donkey => "Donkey",
            Species::Goat => "Goat",
            Species::Sheep => "Sheep",
            Species::Chicken => "Chicken",
            Species::Duck => "Duck",
            Species::Goose => "Goose",
            Species::Rabbit => "Rabbit",
            Species::GuineaPig => "Guinea pig",
            Species::Dog => "Dog",
            Species::Cat => "Cat",
            Species::Other => "Other",
        }
    }

    pub fn family(&self) -> Family {
        match self {
            Species::Horse | Species::Donkey => Family::Equine,
            Species::Goat | Species::Sheep => Family::SmallRuminant,
            Species::Chicken | Species::Duck | Species::Goose => Family::Poultry,
            Species::Rabbit | Species::GuineaPig => Family::SmallMammal,
            Species::Dog => Family::Dog,
            Species::Cat => Family::Cat,
            Species::Other => Family::Other,
        }
    }
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
    /// Bill of sale / adoption contract reference (v1 data lacks it).
    #[serde(default)]
    pub transfer_doc: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Horns {
    Horned,
    Polled,
    Disbudded,
}

impl Horns {
    pub fn label(&self) -> &'static str {
        match self {
            Horns::Horned => "Horned",
            Horns::Polled => "Polled",
            Horns::Disbudded => "Disbudded",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Housing {
    Indoor,
    Outdoor,
    Either,
}

impl Housing {
    pub fn label(&self) -> &'static str {
        match self {
            Housing::Indoor => "Indoor",
            Housing::Outdoor => "Outdoor",
            Housing::Either => "Indoor or outdoor",
        }
    }
}

/// Equine Coggins (EIA) test record.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Coggins {
    pub date: Option<NaiveDate>,
    pub lab: Option<String>,
    pub result: Option<String>,
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
    // Per-species facts; all optional so v1 data still loads.
    #[serde(default)]
    pub height_hands: Option<f32>,
    #[serde(default)]
    pub rideable: YesNoUnknown,
    #[serde(default)]
    pub horns: Option<Horns>,
    #[serde(default)]
    pub layer: YesNoUnknown,
    #[serde(default)]
    pub housing: Option<Housing>,
    #[serde(default)]
    pub good_with_other_animals: YesNoUnknown,
}

/// Times are naive: the web layer supplies local time, core never reads a clock.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Animal {
    pub id: AnimalId,
    pub name: String,
    pub species: Species,
    /// Free-text kind when `species` is Other (e.g. "Llama").
    #[serde(default)]
    pub species_other: Option<String>,
    pub sex: Sex,
    pub received_at: NaiveDateTime,
    pub source: Option<Source>,
    pub out_of_state: bool,
    pub health_certificate: Option<String>,
    /// Proof of ownership / surrender form reference.
    #[serde(default)]
    pub ownership_doc: Option<String>,
    #[serde(default)]
    pub coggins: Option<Coggins>,
    #[serde(default)]
    pub scrapie_tag: Option<String>,
    pub identification: Identification,
    pub health: Vec<HealthEntry>,
    pub story: Story,
    pub photo: Option<String>,
    pub status: Status,
    pub status_changed_at: NaiveDateTime,
    pub disposition: Option<Disposition>,
}

impl Animal {
    /// Lowercase kind for sentences: "goat", "guinea pig", or the free-text kind for Other.
    pub fn noun(&self) -> String {
        match self.species {
            Species::Other => self
                .species_other
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_lowercase)
                .unwrap_or_else(|| "animal".to_string()),
            sp => sp.label().to_lowercase(),
        }
    }

    /// Capitalized kind for display: "Goat", "Llama", "Animal".
    pub fn kind_label(&self) -> String {
        let n = self.noun();
        let mut c = n.chars();
        c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
    }
}

pub const HEIGHT_HANDS_MSG: &str = "Height is hands.inches \u{2014} e.g. 14.2 (the number after the dot is 0\u{2013}3)";

/// Parse a height typed in hands.inches ("14.2" = 14 hands 2 inches). Blank is fine (None);
/// the single digit after the dot must be 0-3 because a hand is 4 inches. No rounding is done.
pub fn parse_height_hands(input: &str) -> Result<Option<f32>, &'static str> {
    let t = input.trim();
    if t.is_empty() {
        return Ok(None);
    }
    let (whole, frac) = t.split_once('.').unwrap_or((t, ""));
    let digits = |x: &str| x.chars().all(|c| c.is_ascii_digit());
    if whole.is_empty() || !digits(whole) || !digits(frac) || frac.len() > 1 || (t.contains('.') && frac.is_empty()) || frac.chars().any(|c| c > '3') {
        return Err(HEIGHT_HANDS_MSG);
    }
    t.parse::<f32>().map(Some).map_err(|_| HEIGHT_HANDS_MSG)
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
            hashtags: vec!["#FarmAnimalRescue".into(), "#AdoptDontShop".into(), "#NebraskaRescue".into()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bare(species: Species, other: Option<&str>) -> Animal {
        let t = NaiveDate::from_ymd_opt(2026, 11, 20).unwrap().and_hms_opt(10, 0, 0).unwrap();
        Animal {
            id: "a".into(),
            name: "X".into(),
            species,
            species_other: other.map(Into::into),
            sex: Sex::Unknown,
            received_at: t,
            source: None,
            out_of_state: false,
            health_certificate: None,
            ownership_doc: None,
            coggins: None,
            scrapie_tag: None,
            identification: Identification::default(),
            health: vec![],
            story: Story::default(),
            photo: None,
            status: Status::Intake,
            status_changed_at: t,
            disposition: None,
        }
    }

    #[test]
    fn height_hands_validator() {
        assert_eq!(parse_height_hands(""), Ok(None));
        assert_eq!(parse_height_hands(" 14.2 "), Ok(Some(14.2)));
        assert_eq!(parse_height_hands("15"), Ok(Some(15.0)));
        assert_eq!(parse_height_hands("14.0"), Ok(Some(14.0)));
        assert_eq!(parse_height_hands("14.3"), Ok(Some(14.3)));
        for bad in ["14.4", "14.9", "14.25", "-1", ".2", "abc", "14.", "1e1"] {
            assert_eq!(parse_height_hands(bad), Err(HEIGHT_HANDS_MSG), "{bad}");
        }
    }

    #[test]
    fn noun_and_label() {
        let a = bare(Species::GuineaPig, None);
        assert_eq!((a.noun(), a.kind_label()), ("guinea pig".into(), "Guinea pig".into()));
        let a = bare(Species::Other, Some("Llama"));
        assert_eq!((a.noun(), a.kind_label()), ("llama".into(), "Llama".into()));
        let a = bare(Species::Other, Some("  "));
        assert_eq!((a.noun(), a.kind_label()), ("animal".into(), "Animal".into()));
        assert_eq!(Species::GuineaPig.label(), "Guinea pig");
        assert_eq!(Species::ALL.len(), 12);
    }

    #[test]
    fn family_mapping() {
        use Family::*;
        let f = |s: Species| s.family();
        assert_eq!((f(Species::Horse), f(Species::Donkey)), (Equine, Equine));
        assert_eq!((f(Species::Goat), f(Species::Sheep)), (SmallRuminant, SmallRuminant));
        for s in [Species::Chicken, Species::Duck, Species::Goose] {
            assert_eq!(f(s), Poultry);
        }
        assert_eq!((f(Species::Rabbit), f(Species::GuineaPig)), (SmallMammal, SmallMammal));
        assert_eq!((f(Species::Dog), f(Species::Cat), f(Species::Other)), (Dog, Cat, Other));
    }
}
