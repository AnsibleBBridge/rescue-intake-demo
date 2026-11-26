//! Adoption post text and fact-sheet lines, built purely from an animal's records.
use crate::model::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tone {
    Upbeat,
    Heartfelt,
    Short,
}

impl Tone {
    pub const ALL: [Tone; 3] = [Tone::Upbeat, Tone::Heartfelt, Tone::Short];

    pub fn label(&self) -> &'static str {
        match self {
            Tone::Upbeat => "Upbeat",
            Tone::Heartfelt => "Heartfelt",
            Tone::Short => "Short",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Post {
    pub tone: Tone,
    pub text: String,
}

/// Subject / object / possessive pronouns chosen from the recorded sex.
struct Pronouns {
    subj: &'static str,
    obj: &'static str,
    poss: &'static str,
    // Verb forms that must agree with the subject (they are / she is).
    is: &'static str,
    deserves: &'static str,
    gets: &'static str,
}

fn pronouns(sex: Sex) -> Pronouns {
    match sex {
        Sex::Male => Pronouns { subj: "he", obj: "him", poss: "his", is: "is", deserves: "deserves", gets: "gets" },
        Sex::Female => Pronouns { subj: "she", obj: "her", poss: "her", is: "is", deserves: "deserves", gets: "gets" },
        Sex::Unknown => Pronouns { subj: "they", obj: "them", poss: "their", is: "are", deserves: "deserve", gets: "get" },
    }
}

fn cap(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

/// Treat blank strings as absent so templates never emit empty phrases.
fn text(o: &Option<String>) -> Option<&str> {
    o.as_deref().map(str::trim).filter(|s| !s.is_empty())
}

/// Label/value pairs for the fact sheet and share image; unknowns are skipped.
pub fn fact_lines(animal: &Animal) -> Vec<(&'static str, String)> {
    let s = &animal.story;
    let mut out = Vec::new();
    if let Some(b) = text(&s.breed) {
        out.push(("Breed", b.to_string()));
    }
    if let Some(a) = text(&s.age_text) {
        out.push(("Age", a.to_string()));
    }
    match animal.sex {
        Sex::Male => out.push(("Sex", "Male".into())),
        Sex::Female => out.push(("Sex", "Female".into())),
        Sex::Unknown => {}
    }
    if let Some(c) = text(&s.color_markings) {
        out.push(("Color", c.to_string()));
    }
    if let Some(w) = s.weight_lbs {
        out.push(("Weight", format!("{} lbs", w.round() as i64)));
    }
    if let Some(sp) = text(&s.size_potential) {
        out.push(("Size potential", sp.to_string()));
    }
    let yn = |v: YesNoUnknown| match v {
        YesNoUnknown::Yes => Some("Yes".to_string()),
        YesNoUnknown::No => Some("No".to_string()),
        YesNoUnknown::Unknown => None,
    };
    if let Some(v) = yn(s.altered) {
        out.push(("Spayed/neutered", v));
    }
    // Only positive compatibility is advertised.
    let good: Vec<&str> = [
        (s.good_with_dogs, "dogs"),
        (s.good_with_cats, "cats"),
        (s.good_with_kids, "kids"),
    ]
    .iter()
    .filter(|(v, _)| *v == YesNoUnknown::Yes)
    .map(|(_, n)| *n)
    .collect();
    if !good.is_empty() {
        out.push(("Good with", good.join(", ")));
    }
    if let Some(v) = yn(s.house_trained) {
        out.push(("House-trained", v));
    }
    if let Some(e) = s.energy {
        out.push((
            "Energy",
            match e {
                Energy::Low => "Low",
                Energy::Medium => "Medium",
                Energy::High => "High",
            }
            .into(),
        ));
    }
    if let Some(f) = s.adoption_fee {
        out.push(("Fee", format!("${f}")));
    }
    out
}

/// "Contact {rescue}: phone · email · website", dropping blank parts.
fn contact_line(settings: &Settings) -> String {
    let parts: Vec<&str> = [&settings.phone, &settings.email, &settings.website]
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    let name = settings.rescue_name.trim();
    match (name.is_empty(), parts.is_empty()) {
        (_, true) if name.is_empty() => String::new(),
        (_, true) => format!("Contact {name}"),
        (true, false) => format!("Contact: {}", parts.join(" · ")),
        (false, false) => format!("Contact {name}: {}", parts.join(" · ")),
    }
}

fn hashtags(settings: &Settings) -> String {
    settings.hashtags.iter().map(|h| h.trim()).filter(|h| !h.is_empty()).collect::<Vec<_>>().join(" ")
}

/// Join non-empty blocks with blank lines.
fn blocks(parts: Vec<String>) -> String {
    parts.into_iter().filter(|p| !p.trim().is_empty()).collect::<Vec<_>>().join("\n\n")
}

/// "a Lab mix" / "a dog" style noun phrase for the animal.
fn kind_phrase(animal: &Animal) -> String {
    let word = text(&animal.story.breed).map(str::to_string).unwrap_or_else(|| animal.noun());
    let article = if word.chars().next().is_some_and(|c| "aeiouAEIOU".contains(c)) { "an" } else { "a" };
    format!("{article} {word}")
}

/// Shared facts sentence: age, personality, needs, fee.
fn details(animal: &Animal, p: &Pronouns) -> Vec<String> {
    let s = &animal.story;
    let mut v = Vec::new();
    if let Some(a) = text(&s.age_text) {
        v.push(format!("Age: {a}."));
    }
    if let Some(pers) = text(&s.personality) {
        v.push(if pers.ends_with(['.', '!', '?']) { pers.to_string() } else { format!("{pers}.") });
    }
    if let Some((_, g)) = fact_lines(animal).into_iter().find(|(l, _)| *l == "Good with") {
        v.push(format!("{} {} along great with {g}.", cap(p.subj), p.gets));
    }
    if let Some(n) = text(&s.special_needs) {
        v.push(format!("Special note: {n}"));
    }
    v
}

pub fn generate_posts(animal: &Animal, settings: &Settings) -> Vec<Post> {
    let p = pronouns(animal.sex);
    let name = animal.name.trim();
    let contact = contact_line(settings);
    let tags = hashtags(settings);
    let fee = animal.story.adoption_fee.map(|f| format!("Adoption fee: ${f}."));
    let kind = kind_phrase(animal);
    let det = details(animal, &p).join(" ");

    let upbeat = blocks(vec![
        format!("Meet {name}! 🐾 {name} is {kind} who is ready to find {} forever home!", p.poss),
        det.clone(),
        fee.clone().unwrap_or_default(),
        contact.clone(),
        tags.clone(),
    ]);

    let heartfelt = blocks(vec![
        format!("{name} has been waiting patiently for someone to say \"you're home.\" {} {} {kind}, and {} {} a family to love {}.", cap(p.subj), p.is, p.subj, p.deserves, p.obj),
        det,
        fee.unwrap_or_default(),
        format!("If {name} could be part of your story, we would love to hear from you."),
        contact.clone(),
        tags.clone(),
    ]);

    // Short: must fit in 280 chars including hashtags and contact.
    let tail = blocks(vec![contact, tags]);
    let mut head = format!("Meet {name}, {kind} looking for a home! 🐾");
    if let Some(pers) = text(&animal.story.personality) {
        head.push(' ');
        head.push_str(pers);
    }
    let budget = 280usize.saturating_sub(tail.chars().count() + 2);
    if head.chars().count() > budget {
        head = head.chars().take(budget.saturating_sub(1)).collect::<String>().trim_end().to_string() + "…";
    }
    let short = blocks(vec![head, tail]);

    vec![
        Post { tone: Tone::Upbeat, text: upbeat },
        Post { tone: Tone::Heartfelt, text: heartfelt },
        Post { tone: Tone::Short, text: short },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn animal(sex: Sex) -> Animal {
        let t = NaiveDate::from_ymd_opt(2026, 11, 20).unwrap().and_hms_opt(10, 0, 0).unwrap();
        Animal {
            id: "a".into(),
            name: "Biscuit".into(),
            species: Species::Dog,
            species_other: None,
            sex,
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
            status: Status::Available,
            status_changed_at: t,
            disposition: None,
        }
    }

    fn all_text(a: &Animal, s: &Settings) -> Vec<String> {
        generate_posts(a, s).into_iter().map(|p| p.text).collect()
    }

    #[test]
    fn sparse_animal_has_no_none_or_double_spaces() {
        for t in all_text(&animal(Sex::Unknown), &Settings::default()) {
            for bad in ["None", "Unknown", "  ", "$"] {
                assert!(!t.contains(bad), "{bad:?} in {t:?}");
            }
        }
    }

    #[test]
    fn includes_name_breed_and_fee_when_present() {
        let mut a = animal(Sex::Male);
        a.story.breed = Some("Lab mix".into());
        a.story.adoption_fee = Some(150);
        let posts = generate_posts(&a, &Settings::default());
        for p in posts.iter().filter(|p| p.tone != Tone::Short) {
            assert!(p.text.contains("Biscuit") && p.text.contains("Lab mix") && p.text.contains("$150"), "{}", p.text);
        }
    }

    #[test]
    fn short_post_within_280() {
        let mut a = animal(Sex::Male);
        a.story.personality = Some("A very long personality description. ".repeat(20));
        let short = generate_posts(&a, &Settings::default()).into_iter().find(|p| p.tone == Tone::Short).unwrap();
        assert!(short.text.chars().count() <= 280, "{}", short.text.chars().count());
        assert!(short.text.contains("#AdoptDontShop"));
    }

    #[test]
    fn contact_line_drops_empty_website() {
        let mut s = Settings { rescue_name: "Prairie Paws".into(), ..Settings::default() };
        assert_eq!(contact_line(&s), "Contact Prairie Paws: (402) 555-0100 · adopt@example.org");
        s.website = "prairiepaws.org".into();
        assert!(contact_line(&s).ends_with("· prairiepaws.org"));
    }

    #[test]
    fn female_uses_she() {
        let mut a = animal(Sex::Female);
        a.story.good_with_dogs = YesNoUnknown::Yes;
        let t = all_text(&a, &Settings::default()).join("\n");
        assert!(t.contains(" her ") && !t.contains(" they "), "{t}");
        assert!(t.contains("She is") && t.contains("she deserves") && t.contains("She gets"), "{t}");
    }

    #[test]
    fn unknown_sex_uses_they() {
        let mut a = animal(Sex::Unknown);
        a.story.good_with_dogs = YesNoUnknown::Yes;
        let t = all_text(&a, &Settings::default()).join("\n");
        assert!(t.contains(" their ") && !t.contains(" his ") && !t.contains(" her "), "{t}");
        for bad in ["They is", "they is", "they deserves", "They gets", "they gets"] {
            assert!(!t.contains(bad), "{bad:?} in {t}");
        }
        assert!(t.contains("They are") && t.contains("they deserve") && t.contains("They get "), "{t}");
    }

    #[test]
    fn fact_lines_skip_unknown_good_with() {
        let mut a = animal(Sex::Male);
        assert!(fact_lines(&a).iter().all(|(l, _)| *l != "Good with" && *l != "Spayed/neutered"));
        a.story.good_with_dogs = YesNoUnknown::Yes;
        a.story.good_with_cats = YesNoUnknown::No;
        a.story.good_with_kids = YesNoUnknown::Yes;
        a.story.altered = YesNoUnknown::Yes;
        a.story.age_text = Some("~2 years".into());
        let f = fact_lines(&a);
        assert!(f.contains(&("Good with", "dogs, kids".to_string())));
        assert!(f.contains(&("Spayed/neutered", "Yes".to_string())));
        assert!(f.contains(&("Age", "~2 years".to_string())));
    }

    #[test]
    fn fact_lines_include_size_potential_when_not_blank() {
        let mut a = animal(Sex::Male);
        a.story.size_potential = Some("   ".into());
        assert!(fact_lines(&a).iter().all(|(l, _)| *l != "Size potential"));
        a.story.size_potential = Some("about 50 lbs grown".into());
        assert!(fact_lines(&a).contains(&("Size potential", "about 50 lbs grown".to_string())));
    }
}
