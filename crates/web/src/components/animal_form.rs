//! Reusable animal form (intake now, edit later). Owns all field signals; on submit it
//! validates the minimum, rebuilds an `Animal` from `initial`, and hands it to `on_save`.
use crate::clock;
use crate::components::{hint::Hint, photo_input::PhotoInput, yes_no::YesNo};
use crate::pages::intake::validate_minimum;
use chrono::{NaiveDate, NaiveDateTime};
use leptos::prelude::*;
use rescue_core::model::*;

/// Shown instead of the 48-hour note for anything that isn't a dog or cat.
pub const FARM_NOTE: &str = "Nebraska has no single farm-sanctuary records law \u{2014} these are reminders, not deadlines. Dogs and cats follow 23 NAC 18.";

const DT_FMT: &str = "%Y-%m-%dT%H:%M";

/// Trimmed text, with blanks stored as None (never Some("")).
fn opt(s: &str) -> Option<String> {
    let t = s.trim();
    (!t.is_empty()).then(|| t.to_string())
}

fn opt_sig(v: &Option<String>) -> RwSignal<String> {
    RwSignal::new(v.clone().unwrap_or_default())
}

/// One editable health row; signals so each input updates without rebuilding the list.
#[derive(Clone, Copy)]
struct HealthRow {
    key: u32,
    date: RwSignal<String>,
    kind: RwSignal<HealthKind>,
    description: RwSignal<String>,
    reason: RwSignal<String>,
    result: RwSignal<String>,
    rabies: RwSignal<String>,
}

impl HealthRow {
    fn new(key: u32, e: &HealthEntry) -> Self {
        HealthRow {
            key,
            date: RwSignal::new(e.date.format("%Y-%m-%d").to_string()),
            kind: RwSignal::new(e.kind),
            description: RwSignal::new(e.description.clone()),
            reason: opt_sig(&e.reason),
            result: opt_sig(&e.result),
            rabies: opt_sig(&e.rabies_certificate),
        }
    }
}

const KINDS: [(HealthKind, &str); 5] = [
    (HealthKind::Vaccination, "Vaccination"),
    (HealthKind::RabiesVaccination, "Rabies vaccination"),
    (HealthKind::Treatment, "Treatment"),
    (HealthKind::Medication, "Medication"),
    (HealthKind::Procedure, "Procedure"),
];

#[component]
pub fn Text(
    label: &'static str,
    value: RwSignal<String>,
    #[prop(optional)] placeholder: &'static str,
    #[prop(default = "text")] kind: &'static str,
) -> impl IntoView {
    view! {
        <label class="field"><span class="lbl">{label}</span>
            <input type=kind placeholder=placeholder prop:value=move || value.get()
                on:input=move |ev| value.set(event_target_value(&ev)) />
        </label>
    }
}

#[component]
pub fn Area(label: &'static str, value: RwSignal<String>, #[prop(optional)] placeholder: &'static str) -> impl IntoView {
    view! {
        <label class="field"><span class="lbl">{label}</span>
            <textarea rows="3" placeholder=placeholder prop:value=move || value.get()
                on:input=move |ev| value.set(event_target_value(&ev)) />
        </label>
    }
}

/// Single-choice chips over an optional value (None is the "Not sure" chip).
#[component]
fn Choice<T: Copy + PartialEq + Send + Sync + 'static>(
    label: &'static str,
    value: RwSignal<Option<T>>,
    opts: Vec<(Option<T>, &'static str)>,
) -> impl IntoView {
    view! {
        <div class="field" role="group" aria-label=label>
            <span class="lbl">{label}</span>
            <div class="chips tight">
                {opts.into_iter().map(|(v, t)| view! {
                    <button type="button" class="chip" class:active=move || value.get() == v
                        aria-pressed=move || (value.get() == v).to_string()
                        on:click=move |_| value.set(v)>{t}</button>
                }).collect_view()}
            </div>
        </div>
    }
}

/// Drop per-species values that don't apply to the chosen species, so switching species in the
/// form (goat -> duck) never saves a stale scrapie tag, horns, Coggins, etc.
pub fn clear_inapplicable(a: &mut Animal) {
    let fam = a.species.family();
    if a.species != Species::Other {
        a.species_other = None;
    }
    if fam != Family::Equine {
        a.coggins = None;
        a.story.height_hands = None;
        a.story.rideable = YesNoUnknown::Unknown;
    }
    if fam != Family::SmallRuminant {
        a.scrapie_tag = None;
        a.story.horns = None;
    }
    if fam != Family::Poultry {
        a.story.layer = YesNoUnknown::Unknown;
    }
    if !has_housing(fam) {
        a.story.housing = None;
    }
    if !matches!(a.species, Species::Dog | Species::Cat | Species::Rabbit) {
        a.story.house_trained = YesNoUnknown::Unknown;
    }
}

/// Families where indoor/outdoor housing is worth asking (small mammals, poultry, goats/sheep).
fn has_housing(f: Family) -> bool {
    matches!(f, Family::SmallMammal | Family::Poultry | Family::SmallRuminant)
}

/// `save_error` lets the caller show a failed save (e.g. storage full) above the submit
/// button without losing the form contents.
#[component]
pub fn AnimalForm(
    initial: Animal,
    on_save: Callback<Animal>,
    submit_label: &'static str,
    #[prop(optional)] save_error: Option<RwSignal<Option<String>>>,
) -> impl IntoView {
    let save_error = save_error.unwrap_or_else(|| RwSignal::new(None));
    let form_error = RwSignal::new(None::<String>);

    // Animal section. A blank (unnamed) animal starts with no species so it must be chosen.
    let name = RwSignal::new(initial.name.clone());
    let species = RwSignal::new((!initial.name.trim().is_empty()).then_some(initial.species));
    let sex = RwSignal::new(initial.sex);
    let species_other = opt_sig(&initial.species_other);
    let received = RwSignal::new(initial.received_at.format(DT_FMT).to_string());
    // Which per-species sections to show; None until a species is chosen.
    let fam = move || species.get().map(|s| s.family());
    let regulated = move || matches!(fam(), Some(Family::Dog | Family::Cat));

    // Source section.
    let is_agency = RwSignal::new(matches!(initial.source, Some(Source::Agency { .. })));
    let (mut p_name, mut p_addr, mut p_dl) = (String::new(), String::new(), String::new());
    let (mut a_name, mut a_title, mut a_addr) = (String::new(), String::new(), String::new());
    match &initial.source {
        Some(Source::Person { name, address, drivers_license }) => {
            (p_name, p_addr, p_dl) = (name.clone(), address.clone(), drivers_license.clone())
        }
        Some(Source::Agency { agency, official_title, address }) => {
            (a_name, a_title, a_addr) = (agency.clone(), official_title.clone(), address.clone())
        }
        None => {}
    }
    let (p_name, p_addr, p_dl) = (RwSignal::new(p_name), RwSignal::new(p_addr), RwSignal::new(p_dl));
    let (a_name, a_title, a_addr) = (RwSignal::new(a_name), RwSignal::new(a_title), RwSignal::new(a_addr));
    let out_of_state = RwSignal::new(initial.out_of_state);
    let health_cert = opt_sig(&initial.health_certificate);
    let ownership = opt_sig(&initial.ownership_doc);
    let scrapie = opt_sig(&initial.scrapie_tag);
    let cog = initial.coggins.clone().unwrap_or_default();
    let cog_date = RwSignal::new(cog.date.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default());
    let cog_lab = opt_sig(&cog.lab);
    let cog_result = opt_sig(&cog.result);

    // Identification section.
    let tag = opt_sig(&initial.identification.tag);
    let cage = opt_sig(&initial.identification.cage_card);
    let tattoo = opt_sig(&initial.identification.tattoo);
    let chip = initial.identification.microchip.clone();
    let chip_no = RwSignal::new(chip.as_ref().map(|c| c.number.clone()).unwrap_or_default());
    let chip_mfr = opt_sig(&chip.as_ref().and_then(|c| c.manufacturer.clone()));
    let chip_loc = opt_sig(&chip.as_ref().and_then(|c| c.location.clone()));

    // Health section.
    let next_key = StoredValue::new(initial.health.len() as u32);
    let rows = RwSignal::new(
        initial.health.iter().enumerate().map(|(i, e)| HealthRow::new(i as u32, e)).collect::<Vec<_>>(),
    );

    // Story section.
    let st = &initial.story;
    let breed = opt_sig(&st.breed);
    let age = opt_sig(&st.age_text);
    let birth = RwSignal::new(st.birth_date.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default());
    let color = opt_sig(&st.color_markings);
    let weight = RwSignal::new(st.weight_lbs.map(|w| w.to_string()).unwrap_or_default());
    let size_potential = opt_sig(&st.size_potential);
    let altered = RwSignal::new(st.altered);
    let dogs = RwSignal::new(st.good_with_dogs);
    let cats = RwSignal::new(st.good_with_cats);
    let kids = RwSignal::new(st.good_with_kids);
    let house = RwSignal::new(st.house_trained);
    let other_animals = RwSignal::new(st.good_with_other_animals);
    let height = RwSignal::new(st.height_hands.map(|h| h.to_string()).unwrap_or_default());
    let rideable = RwSignal::new(st.rideable);
    let horns = RwSignal::new(st.horns);
    let layer = RwSignal::new(st.layer);
    let housing = RwSignal::new(st.housing);
    let energy = RwSignal::new(st.energy);
    let personality = opt_sig(&st.personality);
    let special = opt_sig(&st.special_needs);
    let fee = RwSignal::new(st.adoption_fee.map(|f| f.to_string()).unwrap_or_default());

    let photo = RwSignal::new(initial.photo.clone());

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        save_error.set(None);
        let received_at = NaiveDateTime::parse_from_str(&received.get_untracked(), DT_FMT).ok();
        if let Err(missing) = validate_minimum(&name.get_untracked(), species.get_untracked(), received_at) {
            form_error.set(Some(format!("Please fill in: {}", missing.join(", "))));
            return;
        }
        // Half-filled health rows would be silently lost, so refuse them instead.
        let mut health = Vec::new();
        for (i, r) in rows.get_untracked().iter().enumerate() {
            let date = NaiveDate::parse_from_str(&r.date.get_untracked(), "%Y-%m-%d").ok();
            let desc = r.description.get_untracked();
            let (Some(date), Some(description)) = (date, opt(&desc)) else {
                form_error.set(Some(format!("Health entry {} needs a date and a description (or remove it).", i + 1)));
                return;
            };
            let kind = r.kind.get_untracked();
            health.push(HealthEntry {
                date,
                kind,
                description,
                reason: opt(&r.reason.get_untracked()),
                result: opt(&r.result.get_untracked()),
                rabies_certificate: if kind == HealthKind::RabiesVaccination { opt(&r.rabies.get_untracked()) } else { None },
            });
        }
        form_error.set(None);

        let source = if is_agency.get_untracked() {
            let (a, t, ad) = (a_name.get_untracked(), a_title.get_untracked(), a_addr.get_untracked());
            [&a, &t, &ad].iter().any(|s| !s.trim().is_empty()).then(|| Source::Agency {
                agency: a.trim().into(),
                official_title: t.trim().into(),
                address: ad.trim().into(),
            })
        } else {
            let (n, ad, dl) = (p_name.get_untracked(), p_addr.get_untracked(), p_dl.get_untracked());
            [&n, &ad, &dl].iter().any(|s| !s.trim().is_empty()).then(|| Source::Person {
                name: n.trim().into(),
                address: ad.trim().into(),
                drivers_license: dl.trim().into(),
            })
        };
        let microchip = opt(&chip_no.get_untracked()).map(|number| Microchip {
            number,
            manufacturer: opt(&chip_mfr.get_untracked()),
            location: opt(&chip_loc.get_untracked()),
        });

        let mut a = initial.clone();
        a.name = name.get_untracked().trim().to_string();
        a.species = species.get_untracked().unwrap_or(initial.species);
        a.species_other = opt(&species_other.get_untracked());
        a.sex = sex.get_untracked();
        a.received_at = received_at.unwrap_or(initial.received_at);
        a.source = source;
        a.out_of_state = out_of_state.get_untracked();
        a.health_certificate = if a.out_of_state { opt(&health_cert.get_untracked()) } else { None };
        a.ownership_doc = opt(&ownership.get_untracked());
        a.scrapie_tag = opt(&scrapie.get_untracked());
        let coggins = Coggins {
            date: NaiveDate::parse_from_str(&cog_date.get_untracked(), "%Y-%m-%d").ok(),
            lab: opt(&cog_lab.get_untracked()),
            result: opt(&cog_result.get_untracked()),
        };
        a.coggins = (coggins != Coggins::default()).then_some(coggins);
        a.identification = Identification {
            tag: opt(&tag.get_untracked()),
            cage_card: opt(&cage.get_untracked()),
            tattoo: opt(&tattoo.get_untracked()),
            microchip,
        };
        a.health = health;
        a.story = Story {
            breed: opt(&breed.get_untracked()),
            age_text: opt(&age.get_untracked()),
            birth_date: NaiveDate::parse_from_str(&birth.get_untracked(), "%Y-%m-%d").ok(),
            color_markings: opt(&color.get_untracked()),
            weight_lbs: weight.get_untracked().trim().parse().ok(),
            size_potential: opt(&size_potential.get_untracked()),
            altered: altered.get_untracked(),
            good_with_dogs: dogs.get_untracked(),
            good_with_cats: cats.get_untracked(),
            good_with_kids: kids.get_untracked(),
            house_trained: house.get_untracked(),
            energy: energy.get_untracked(),
            personality: opt(&personality.get_untracked()),
            special_needs: opt(&special.get_untracked()),
            adoption_fee: fee.get_untracked().trim().parse().ok(),
            height_hands: height.get_untracked().trim().parse().ok(),
            rideable: rideable.get_untracked(),
            horns: horns.get_untracked(),
            layer: layer.get_untracked(),
            housing: housing.get_untracked(),
            good_with_other_animals: other_animals.get_untracked(),
        };
        a.photo = photo.get_untracked();
        // Fields hidden for this species must not keep values typed before switching.
        clear_inapplicable(&mut a);
        on_save.run(a);
    };

    let add_row = move |_| {
        let key = next_key.get_value();
        next_key.set_value(key + 1);
        let blank = HealthEntry {
            date: clock::now().date(),
            kind: HealthKind::Vaccination,
            description: String::new(),
            reason: None,
            result: None,
            rabies_certificate: None,
        };
        rows.update(|r| r.push(HealthRow::new(key, &blank)));
    };

    view! {
        <form class="form" on:submit=on_submit novalidate>
            <p class="note">{move || if regulated() {
                "Only name, species and date received are required now \u{2014} Nebraska gives you 48 hours to complete the rest (23 NAC 18-010.04A).".to_string()
            } else {
                format!("Only name, species and date received are required now. {FARM_NOTE}")
            }}</p>
            <p class="note">"This is a demo \u{2014} please use made-up names, addresses and license numbers."</p>

            <section class="card">
                <h2>"Animal"</h2>
                <Text label="Name" value=name />
                <div class="field"><label class="lbl" for="species">"Species"</label>
                    <select id="species" on:change=move |ev| {
                        let v = event_target_value(&ev);
                        species.set(Species::ALL.iter().copied().find(|sp| sp.label() == v));
                    }>
                        <option value="" selected=move || species.get().is_none()>"Choose a species\u{2026}"</option>
                        {Species::ALL.iter().map(|sp| view! {
                            <option value=sp.label() selected=move || species.get() == Some(*sp)>{sp.label()}</option>
                        }).collect_view()}
                    </select>
                </div>
                {move || (species.get() == Some(Species::Other)).then(|| view! {
                    <Text label="What kind of animal?" value=species_other placeholder="e.g. alpaca" />
                })}
                <div class="field" role="group" aria-label="Sex">
                    <span class="lbl">"Sex"</span>
                    <div class="chips tight">
                        {[(Sex::Male, "Male"), (Sex::Female, "Female"), (Sex::Unknown, "Unknown")].into_iter().map(|(v, t)| view! {
                            <button type="button" class="chip" class:active=move || sex.get() == v
                                aria-pressed=move || (sex.get() == v).to_string()
                                on:click=move |_| sex.set(v)>{t}</button>
                        }).collect_view()}
                    </div>
                </div>
                <Text label="Date and time received" value=received kind="datetime-local" />
            </section>

            <section class="card">
                <h2>"Identification"</h2>
                <p class="sub">"Any one of these identifies the animal. Each animal needs its own ID."</p>
                <Text label="Tag #" value=tag />
                <Text label="Cage card #" value=cage />
                <Text label="Tattoo" value=tattoo />
                <Text label="Microchip #" value=chip_no />
                {move || (!chip_no.get().trim().is_empty()).then(|| view! {
                    <Text label="Microchip manufacturer" value=chip_mfr />
                    <Text label="Microchip location" value=chip_loc placeholder="e.g. between the shoulder blades" />
                    <Hint section="23 NAC 18-008.07" text="Microchip records include the number, manufacturer and where it was implanted." />
                })}
                {move || (fam() == Some(Family::SmallRuminant)).then(|| view! {
                    <Text label="Scrapie tag #" value=scrapie />
                    <Hint section="NDA scrapie ID rules (9 CFR 79)" text="Sheep and goats usually need an official scrapie tag before they change hands." />
                })}
                {move || if regulated() {
                    view! { <Hint section="23 NAC 18-008" text="Every animal must be individually identified. Litters kept with their mother may share her ID until they are separated." /> }.into_any()
                } else {
                    view! { <Hint section="Good practice" text="Any one ID (tag, band, tattoo, chip or a pen/cage card) lets you tell animals apart in your records." /> }.into_any()
                }}
            </section>

            <section class="card">
                <h2>"Source"</h2>
                <div class="field" role="group" aria-label="Received from">
                    <span class="lbl">"Received from"</span>
                    <div class="chips tight">
                        <button type="button" class="chip" class:active=move || !is_agency.get()
                            aria-pressed=move || (!is_agency.get()).to_string()
                            on:click=move |_| is_agency.set(false)>"Person"</button>
                        <button type="button" class="chip" class:active=move || is_agency.get()
                            aria-pressed=move || is_agency.get().to_string()
                            on:click=move |_| is_agency.set(true)>"Government agency"</button>
                    </div>
                </div>
                {move || if is_agency.get() {
                    view! {
                        <Text label="Agency name" value=a_name />
                        <Text label="Official's title" value=a_title />
                        <Area label="Mailing address" value=a_addr />
                    }.into_any()
                } else {
                    view! {
                        <Text label="Name" value=p_name />
                        <Area label="Mailing address" value=p_addr />
                        <Text label="Driver's license #" value=p_dl />
                    }.into_any()
                }}
                <Text label="Proof of ownership / surrender form" value=ownership placeholder="e.g. form number or brand inspection #" />
                {move || (!regulated()).then(|| view! {
                    <Hint section="Nebraska Brand Committee (proof of ownership)" text="Keep the surrender form or other proof the animal was legally yours to take in; it protects you if ownership is ever questioned." />
                })}
                {move || {
                    let text = if is_agency.get() {
                        "Nebraska requires the agency's name, the official's title and its mailing address (23 NAC 18-010.04A(3))."
                    } else {
                        "Nebraska requires the name, full mailing address and driver's license number of the person you received the animal from."
                    };
                    if regulated() {
                        view! { <Hint section="23 NAC 18-010.04A(3)" text=text /> }.into_any()
                    } else {
                        view! { <Hint section="Good practice" text="Write down who the animal came from and where, so you can follow up on its history later." /> }.into_any()
                    }
                }}
                <label class="check">
                    <input type="checkbox" prop:checked=move || out_of_state.get()
                        on:change=move |ev| out_of_state.set(event_target_checked(&ev)) />
                    <span>"Out of state?"</span>
                </label>
                {move || out_of_state.get().then(|| view! {
                    <Text label="Certificate of veterinary inspection #" value=health_cert />
                    {if regulated() {
                        view! { <Hint section="23 NAC 18-010.04A(5)" text="Animals brought in from out of state need a health certificate on file." /> }.into_any()
                    } else {
                        view! { <Hint section="NDA import requirements" text="Animals brought in from out of state usually need a certificate of veterinary inspection." /> }.into_any()
                    }}
                })}
            </section>

            <section class="card">
                <h2>"Health"</h2>
                <For each=move || rows.get() key=|r| r.key children=move |r| {
                    let n = move || rows.get().iter().position(|x| x.key == r.key).map(|i| i + 1).unwrap_or(0);
                    view! {
                        <div class="health-row">
                            <strong>{move || format!("Entry {}", n())}</strong>
                            <Text label="Date" value=r.date kind="date" />
                            <label class="field"><span class="lbl">"Type"</span>
                                <select on:change=move |ev| {
                                    let v = event_target_value(&ev);
                                    if let Some((k, _)) = KINDS.iter().find(|(_, l)| *l == v) { r.kind.set(*k); }
                                }>
                                    {KINDS.iter().map(|(k, l)| view! {
                                        <option value=*l selected=move || r.kind.get() == *k>{*l}</option>
                                    }).collect_view()}
                                </select>
                            </label>
                            <Text label="Description" value=r.description />
                            <Text label="Reason" value=r.reason />
                            <Text label="Result" value=r.result />
                            {move || (r.kind.get() == HealthKind::RabiesVaccination).then(|| view! {
                                <Text label="Rabies certificate #" value=r.rabies />
                            })}
                            <button type="button" class="btn" on:click=move |_| rows.update(|v| v.retain(|x| x.key != r.key))>"Remove entry"</button>
                        </div>
                    }
                } />
                <button type="button" class="btn" on:click=add_row>"Add health entry"</button>
                {move || (fam() == Some(Family::Equine)).then(|| view! {
                    <h3>"Coggins (EIA) test"</h3>
                    <Text label="Coggins date" value=cog_date kind="date" />
                    <Text label="Coggins lab" value=cog_lab />
                    <Text label="Coggins result" value=cog_result placeholder="e.g. negative" />
                    <Hint section="NDA import requirements" text="Horses and donkeys coming from out of state usually need a current negative Coggins test; keep a copy either way." />
                })}
                {move || if regulated() {
                    view! { <Hint section="23 NAC 18-010.06" text="Records must list each vaccination, treatment, medication or procedure with its date, reason and result." /> }.into_any()
                } else {
                    view! { <Hint section="Good practice" text="Note each vaccination, treatment or vet visit with its date so the next caretaker knows the history." /> }.into_any()
                }}
            </section>

            <section class="card">
                <h2>"Story"</h2>
                <Text label="Breed" value=breed />
                <Text label="Age" value=age placeholder="e.g. about 2 years" />
                <Text label="Birth date (if known)" value=birth kind="date" />
                <Text label="Color / markings" value=color />
                <Text label="Weight (lbs)" value=weight kind="number" />
                <Text label="Size potential" value=size_potential placeholder="e.g. about 50 lbs grown" />
                {move || (fam() == Some(Family::Equine)).then(|| view! {
                    <Text label="Height (hands)" value=height kind="number" placeholder="e.g. 14.2" />
                    <YesNo value=rideable label="Rideable" />
                })}
                {move || (fam() == Some(Family::SmallRuminant)).then(|| view! {
                    <Choice label="Horns" value=horns opts=vec![
                        (None, "Not sure"), (Some(Horns::Horned), "Horned"), (Some(Horns::Polled), "Polled"), (Some(Horns::Disbudded), "Disbudded"),
                    ] />
                })}
                {move || (fam() == Some(Family::Poultry)).then(|| view! { <YesNo value=layer label="Lays eggs" /> })}
                {move || fam().is_some_and(has_housing).then(|| view! {
                    <Choice label="Housing" value=housing opts=vec![
                        (None, "Not sure"), (Some(Housing::Indoor), "Indoor"), (Some(Housing::Outdoor), "Outdoor"), (Some(Housing::Either), "Either"),
                    ] />
                })}
                <YesNo value=altered label="Spayed / neutered" />
                <YesNo value=dogs label="Good with dogs" />
                <YesNo value=cats label="Good with cats" />
                <YesNo value=kids label="Good with kids" />
                <YesNo value=other_animals label="Good with other animals" />
                {move || matches!(species.get(), Some(Species::Dog | Species::Cat | Species::Rabbit)).then(|| view! {
                    <YesNo value=house label="House-trained" />
                })}
                <Choice label="Energy" value=energy opts=vec![
                    (None, "Not sure"), (Some(Energy::Low), "Low"), (Some(Energy::Medium), "Medium"), (Some(Energy::High), "High"),
                ] />
                <Area label="Personality" value=personality />
                <Area label="Special needs" value=special />
                <Text label="Adoption fee ($)" value=fee kind="number" />
            </section>

            <section class="card">
                <h2>"Photo"</h2>
                <PhotoInput value=photo />
            </section>

            {move || form_error.get().map(|e| view! { <p class="notice" role="alert">{e}</p> })}
            {move || save_error.get().map(|e| view! { <p class="notice" role="alert">{e}</p> })}
            <button type="submit" class="btn primary wide">{submit_label}</button>
        </form>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A goat with every per-species field filled in, as if the user typed them all.
    fn loaded(species: Species) -> Animal {
        let t = NaiveDate::from_ymd_opt(2026, 11, 20).unwrap().and_hms_opt(10, 0, 0).unwrap();
        Animal {
            id: "a".into(), name: "X".into(), species, species_other: Some("Llama".into()), sex: Sex::Unknown,
            received_at: t, source: None, out_of_state: false, health_certificate: None, ownership_doc: Some("SF-1".into()),
            coggins: Some(Coggins { date: Some(t.date()), lab: None, result: None }),
            scrapie_tag: Some("NE-1".into()), identification: Identification::default(), health: vec![],
            story: Story {
                height_hands: Some(14.0), rideable: YesNoUnknown::Yes, horns: Some(Horns::Horned), layer: YesNoUnknown::Yes,
                housing: Some(Housing::Indoor), house_trained: YesNoUnknown::Yes, good_with_other_animals: YesNoUnknown::Yes,
                ..Default::default()
            },
            photo: None, status: Status::Intake, status_changed_at: t, disposition: None,
        }
    }

    #[test]
    fn switching_goat_to_duck_drops_goat_fields() {
        let mut a = loaded(Species::Duck);
        clear_inapplicable(&mut a);
        assert_eq!((a.scrapie_tag, a.coggins, a.species_other), (None, None, None));
        assert_eq!((a.story.horns, a.story.height_hands, a.story.rideable), (None, None, YesNoUnknown::Unknown));
        // Duck keeps what applies to it; house-training is dropped.
        assert_eq!((a.story.layer, a.story.housing), (YesNoUnknown::Yes, Some(Housing::Indoor)));
        assert_eq!(a.story.house_trained, YesNoUnknown::Unknown);
        assert_eq!(a.ownership_doc.as_deref(), Some("SF-1"));
        assert_eq!(a.story.good_with_other_animals, YesNoUnknown::Yes);
    }

    #[test]
    fn horse_keeps_equine_fields_and_dog_keeps_house_training() {
        let mut h = loaded(Species::Horse);
        clear_inapplicable(&mut h);
        assert!(h.coggins.is_some() && h.story.height_hands.is_some() && h.scrapie_tag.is_none());
        assert_eq!((h.story.layer, h.story.housing), (YesNoUnknown::Unknown, None));
        let mut d = loaded(Species::Dog);
        clear_inapplicable(&mut d);
        assert_eq!(d.story.house_trained, YesNoUnknown::Yes);
        let mut o = loaded(Species::Other);
        clear_inapplicable(&mut o);
        assert_eq!(o.species_other.as_deref(), Some("Llama"));
    }
}
