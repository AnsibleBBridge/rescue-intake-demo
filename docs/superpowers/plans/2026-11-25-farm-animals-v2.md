# Farm Animals v2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the dog/cat demo into a farm-animal rescue demo (species, per-species fields, per-species reminders, wording, samples) without breaking stored v1 data.

**Architecture:** Same workspace. Model + reminder rules + wording + samples change in `rescue-core` (pure, tested); `web` shows species-conditional fields and the renamed "Records & reminders" card. New fields are `#[serde(default)]` so v1 localStorage still loads (SCHEMA_VERSION stays 1).

**Tech Stack:** unchanged (Rust, Leptos 0.8 CSR, Trunk, Playwright).

**Spec:** `docs/superpowers/specs/2026-11-25-rescue-intake-demo-design.md` — section "v2 — Farm animals".

## Global Constraints

- Species order everywhere: Horse, Donkey, Goat, Sheep, Chicken, Duck, Goose, Rabbit, Guinea pig, Dog, Cat, Other.
- v1 stored JSON (species "Dog"/"Cat", no new fields) must deserialize unchanged; SCHEMA_VERSION stays 1.
- Only Dog/Cat items carry 48-hour deadlines (23 NAC 18). All other items are `ItemState::Reminder` when missing (never Overdue), except the scrapie-at-transfer rule below.
- Source strings (exact): `"NDA scrapie ID rules (9 CFR 79)"`, `"NDA import requirements"`, `"Good practice"`, `"Nebraska Brand Committee (proof of ownership)"`; dog/cat items keep their `23 NAC 18-…` cites.
- Default hashtags: `["#FarmAnimalRescue", "#AdoptDontShop", "#NebraskaRescue"]`.
- Never print "None", "Unknown", "Other" (as a species name when `species_other` is set), or double spaces in posts/fact sheet/profile.
- Footer/banner/no-network/relative-URL/core-purity constraints from v1 still bind.

## Review Focus

1. **v1 data in her browser** (Dog/Cat animals, old settings) → loads, renders, no notice. *(Task 1 test with a literal v1 JSON blob)*
2. **Species "Other" with blank kind** → shows "Animal"/"animal", never "Other" or blank. *(Task 1 `noun()` test, Task 3 posts test)*
3. **Switching species in the form** hides irrelevant fields and does not save stale per-species values (e.g. goat → duck drops scrapie tag). *(Task 4 e2e)*
4. **Goat/sheep adopted out with no scrapie tag** → clearly flagged. *(Task 2 test)*
5. **Phone width** for the longer species select + conditional fields. *(Task 4 e2e 375px)*

---

### Task 1: Species + per-species fields in the model

**Files:** Modify `crates/rescue-core/src/model.rs`, `store_data.rs` (tests), `crates/rescue-core/src/samples.rs` (compile only — real sample rewrite is Task 3).

**Interfaces — Produces:**
- `enum Species { Horse, Donkey, Goat, Sheep, Chicken, Duck, Goose, Rabbit, GuineaPig, Dog, Cat, Other }` with `ALL: [Species; 12]`, `label() -> &'static str` ("Guinea pig" for GuineaPig), and `fn family(&self) -> Family` where `enum Family { Equine, SmallRuminant, Poultry, SmallMammal, Dog, Cat, Other }`.
- `impl Animal { fn noun(&self) -> String }` — lowercase kind ("goat", "guinea pig"; Other → trimmed `species_other` lowercased, or "animal" if blank). `fn kind_label(&self) -> String` — capitalized form for display ("Goat", "Llama", "Animal").
- New `Animal` fields (all `#[serde(default)]`): `species_other: Option<String>`, `ownership_doc: Option<String>` (proof of ownership / surrender form reference), `coggins: Option<Coggins>`, `scrapie_tag: Option<String>`. Keep `health_certificate` (now labelled "Certificate of veterinary inspection #" in UI).
- `struct Coggins { date: Option<NaiveDate>, lab: Option<String>, result: Option<String> }`.
- New `Story` fields (`#[serde(default)]`): `height_hands: Option<f32>`, `rideable: YesNoUnknown`, `horns: Option<Horns>` (`enum Horns { Horned, Polled, Disbudded }` + `label()`), `layer: YesNoUnknown`, `housing: Option<Housing>` (`enum Housing { Indoor, Outdoor, Either }` + `label()`), `good_with_other_animals: YesNoUnknown`.
- New `Disposition` field (`#[serde(default)]`): `transfer_doc: Option<String>` (bill of sale / adoption contract reference).
- `Settings::default().hashtags` → the v2 default list.

- [ ] **Step 1: Failing tests:** `v1_blob_still_loads` (a literal v1 JSON string with one Dog animal and v1 settings → `from_json` Ok, species Dog, new fields None/Unknown); `noun_and_label` (GuineaPig → "guinea pig"/"Guinea pig"; Other+"Llama" → "llama"/"Llama"; Other+"  " → "animal"/"Animal"); `family_mapping` (Horse/Donkey→Equine, Goat/Sheep→SmallRuminant, Chicken/Duck/Goose→Poultry, Rabbit/GuineaPig→SmallMammal); `round_trip_with_v2_fields` (coggins + scrapie_tag + horns + transfer_doc survive).
- [ ] **Step 2:** `cargo test -p rescue-core` → FAIL. **Step 3:** implement (fix compile breaks in samples/posts/compliance minimally; web may be fixed in Task 4 — but `cargo build --workspace` must pass at the end of this task, so do minimal web compile fixes too). **Step 4:** PASS. Commit `feat(core): farm species and per-species fields`.

### Task 2: Records & reminders per species

**Files:** Modify `crates/rescue-core/src/compliance.rs`.

**Interfaces:** `check(&Animal, now) -> Vec<ComplianceItem>` unchanged signature. Dog/Cat → exactly today's v1 items (unchanged behavior/tests). Others → items below, Done or `Reminder` (no deadline) unless noted:

| key | label | section | applies | Done when |
|---|---|---|---|---|
| `source_contact` | Where it came from (name & address) | Good practice | non-dog/cat | Person: name+address non-blank; Agency: agency+address non-blank |
| `ownership_doc` | Proof of ownership or surrender form | Nebraska Brand Committee (proof of ownership) | all non-dog/cat | `ownership_doc` non-blank |
| `health_record` | Vet / health notes started | Good practice | non-dog/cat | `health` non-empty |
| `cvi` | Certificate of veterinary inspection (arrived from out of state) | NDA import requirements | non-dog/cat AND out_of_state | `health_certificate` non-blank |
| `coggins` | Coggins (EIA) test on file | NDA import requirements (if out of state) / Good practice | Equine | coggins.date is Some |
| `scrapie_tag` | Official scrapie tag # | NDA scrapie ID rules (9 CFR 79) | SmallRuminant | `scrapie_tag` non-blank. **If status Outcome with method Adopted or Transferred and tag missing → `Overdue{ due: disposition date 00:00 or status_changed_at }`** |
| `outcome_record` | Outcome record (date, who it went to) | Good practice | non-dog/cat, status Outcome | Adopted/Transferred/ReturnedToOwner: date+name+address; Died/Euthanized: date+circumstances (Euthanized also by+method) |
| `transfer_doc` | Bill of sale or adoption contract | Nebraska Brand Committee (proof of ownership) | non-dog/cat, status Outcome with Adopted/Transferred | `disposition.transfer_doc` non-blank |

Coggins `section`: `"NDA import requirements"` when `out_of_state`, else `"Good practice"`.

- [ ] **Step 1: Failing tests:** `dog_items_unchanged` (existing dog tests still pass untouched); `goat_without_tag_is_reminder`; `goat_adopted_without_tag_is_overdue`; `donkey_coggins_section_depends_on_out_of_state`; `duck_has_no_coggins_or_scrapie_items`; `non_dog_items_never_overdue_except_scrapie` (an Equine received 10 days ago with nothing filled → no Overdue); `cvi_only_when_out_of_state`; `transfer_doc_only_for_adopted_or_transferred`.
- [ ] **Step 2–4:** FAIL → implement → PASS. Commit `feat(core): per-species records and reminders`.

### Task 3: Species-aware wording + farm samples + art

**Files:** Modify `crates/rescue-core/src/posts.rs`, `samples.rs`; create `crates/web/assets/samples/{clover,jasper,puddles}.svg` (friendly flat illustrations: goat, donkey, duck; <10 KB; same style as existing biscuit/pepper/moose.svg, which may be deleted along with their references).

**Interfaces — Produces:** `fact_lines` adds (when known): "Height" ("14.2 hands" — format with up to one decimal), "Horns" (label), "Rideable"/"Leads" → use label "Rideable" Yes/No, "Lays eggs" Yes/No (Poultry only), "Housing" (label), "Good with" now includes "other animals"; "House-trained" only for Dog/Cat/Rabbit. Posts use `animal.noun()` ("this sweet donkey", "a goat"), a/an chosen by noun. `sample_animals(now, &[Option<String>;3])` returns Clover (Goat, Available, complete incl. scrapie tag, ownership_doc, health entries), Jasper (Donkey, VetCheck, out_of_state = true with CVI filled, missing coggins + ownership_doc), Puddles (Duck, Outcome/Adopted with complete disposition + transfer_doc). Photos `assets/samples/clover.svg` etc.

- [ ] **Step 1: Failing tests:** `posts_use_species_noun` (Donkey → contains "donkey", not "dog"); `other_species_blank_kind_says_animal` (no post contains "Other" or "None"); `an_before_vowel_noun` ("an alpaca"-style via Other "Alpaca"); `fact_lines_horse_height_and_poultry_layer`; `house_trained_hidden_for_goat`; `samples_farm_states` (species {Goat, Donkey, Duck}; Clover has no Open/Overdue items… Reminder allowed? — **Clover has zero non-Done items**; Jasper has a `coggins` Reminder and `ownership_doc` Reminder; Puddles zero non-Done items).
- [ ] **Step 2–4:** FAIL → implement → PASS. Commit `feat(core): species-aware wording and farm samples`.

### Task 4: Web — species form, reminders card, pages, e2e

**Files:** Modify `crates/web/src/components/animal_form.rs`, `components/compliance_list.rs`, `pages/{profile,roster,fact_sheet,posts}.rs`, `share_image.rs`, placeholder silhouettes (one per `Family`), `style.css`; update all `e2e/tests/*.spec.ts` that reference Biscuit/Pepper/Moose → Clover/Jasper/Puddles.

- Form: species `<select>` in Global order; "Other" reveals "What kind of animal?"; per-family sections shown only when relevant (Equine: Height (hands), Rideable, Coggins date/lab/result; SmallRuminant: Scrapie tag #, Horns; Poultry: Lays eggs; SmallMammal: Housing; House-trained for Dog/Cat/Rabbit); "Good with other animals" for all; "Proof of ownership / surrender form" field in Source for all; out-of-state field labelled "Certificate of veterinary inspection #". On save, per-family fields that don't apply to the chosen species are cleared (None/Unknown). Dog/Cat keep their 23 NAC 18 hints; other hints cite the v2 sources. Top note for non-dog/cat: "Nebraska has no single farm-sanctuary records law — these are reminders, not deadlines. Dogs and cats follow 23 NAC 18."
- Disposition form: "Bill of sale / adoption contract #" field.
- Card title "Records & reminders" everywhere ("Compliance" removed).
- Display: `kind_label()` replaces `species.label()` in roster/profile/sheet; species placeholder by family.
- e2e: update existing specs to the new samples; add `farm.spec.ts`: (1) new goat intake shows Scrapie + Horns fields, not Coggins; switch to Duck → scrapie hidden; save → stored record has `scrapie_tag: null`; (2) Jasper shows Coggins + ownership reminders (ℹ️, not red); (3) Other + "Alpaca" → roster card shows "Alpaca"; posts contain "alpaca"; (4) a v1 localStorage blob (Dog) seeded before load → roster shows the dog, no reset notice; (5) 375px no h-scroll on intake with Equine fields open.
- [ ] Steps: implement → `cargo test --workspace`, clippy 0 warnings, `trunk build`, full e2e green → screenshots → commit `feat(web): farm species, reminders, samples`.
