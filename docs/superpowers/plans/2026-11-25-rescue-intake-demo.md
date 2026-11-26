# Rescue Intake Demo Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A static Leptos (Rust→WASM) demo on GitHub Pages where a Nebraska rescue operator does an intake, walks the animal through statuses, and gets a fact sheet and social posts.

**Architecture:** Cargo workspace with two crates. `rescue-core` is pure Rust (model, compliance rules, post text, sample data, storage schema) with no browser deps — it is the part the future server reuses. `web` is a Leptos CSR app built by Trunk; it renders pages, keeps data in localStorage behind a `Store` trait, and does browser-only work (photo downscale, share-image canvas, print).

**Tech Stack:** Rust stable, `leptos` 0.8 (`csr`), Trunk, `serde`/`serde_json`, `chrono` (no `clock` feature in core), `uuid` (v4, `js` feature in web), `web-sys`/`wasm-bindgen`, GitHub Actions + Pages.

**Spec:** `docs/superpowers/specs/2026-11-25-rescue-intake-demo-design.md`

## Global Constraints

- Repo `AnsibleBBridge/rescue-intake-demo`, public; Trunk `--public-url /rescue-intake-demo/`.
- No network calls at runtime, no analytics, no third-party scripts. Fonts are system fonts.
- Every page footer: "Demo only — not legal advice. Confirm current requirements with the Nebraska Department of Agriculture."
- Persistent banner: "Demo — entries stay in this browser only."
- Submit requires only name, species, date received. Everything else is optional (`Option`) and surfaced by the compliance checklist.
- Statuses: `Intake, VetCheck, MedicalHold, Available, AdoptionPending, Outcome`; free movement between them. Outcome methods: `Adopted, Transferred, ReturnedToOwner, Died, Euthanized`.
- Legal hints cite `23 NAC 18-0xx` sections exactly as in the spec.
- Photos stored as JPEG data URLs, longest side ≤1200px, quality 0.8.
- `rescue-core` must not depend on `web-sys`, `leptos`, or a system clock; callers pass `now: NaiveDateTime`.
- Routing is hash-based (`#/`, `#/new`, `#/animal/<id>`, `#/animal/<id>/sheet`, `#/animal/<id>/posts`, `#/settings`) so refreshes work on Pages.

## Review Focus

1. **Huge phone photo (12 MB HEIC/JPEG)** → downscaled before save; if the browser can't decode it (HEIC on desktop Chrome) show "Couldn't read that photo — try a JPEG or PNG", don't save garbage. *(Task 6)*
2. **localStorage full** → save shows a visible error and keeps the form contents; never silently drops the record. *(Task 5 `LocalStore::save` returns `Result`, UI shows it)*
3. **Corrupt or old-version stored JSON** → app loads the samples and shows "Saved demo data couldn't be read and was reset", no panic. *(Task 1 `StoreData::from_json`, Task 5)*
4. **Sparse animal (no breed, age, photo, personality)** → posts and fact sheet omit those phrases cleanly — never "None", "Unknown unknown", or double spaces; placeholder art where the photo would be. *(Task 3 tests, Task 8/9)*
5. **Phone width (375px) and printing** → forms single-column, no horizontal scroll; fact sheet prints on one Letter page without nav/banner. *(Task 11 verification)*

---

### Task 1: Toolchain, workspace, core model, storage schema

**Files:**
- Create: `Cargo.toml` (workspace: `crates/rescue-core`, `crates/web`), `.gitignore` (`target/`, `dist/`)
- Create: `crates/rescue-core/Cargo.toml`, `src/lib.rs`, `src/model.rs`, `src/store_data.rs`

**Interfaces — Produces (in `rescue_core::model`):**
- `pub type AnimalId = String;`
- `enum Species { Dog, Cat }`, `enum Sex { Male, Female, Unknown }`, `enum YesNoUnknown { Yes, No, Unknown }`, `enum Energy { Low, Medium, High }`
- `enum Status { Intake, VetCheck, MedicalHold, Available, AdoptionPending, Outcome }` with `Status::ALL: [Status; 6]` and `fn label(&self) -> &'static str`
- `enum OutcomeMethod { Adopted, Transferred, ReturnedToOwner, Died, Euthanized }` with `label()`
- `enum Source { Person { name: String, address: String, drivers_license: String }, Agency { agency: String, official_title: String, address: String } }`
- `struct Microchip { number: String, manufacturer: Option<String>, location: Option<String> }`
- `struct Identification { tag: Option<String>, cage_card: Option<String>, tattoo: Option<String>, microchip: Option<Microchip> }` + `fn has_any(&self) -> bool`
- `enum HealthKind { Vaccination, RabiesVaccination, Treatment, Medication, Procedure }`
- `struct HealthEntry { date: NaiveDate, kind: HealthKind, description: String, reason: Option<String>, result: Option<String>, rabies_certificate: Option<String> }`
- `struct Disposition { date: Option<NaiveDate>, method: OutcomeMethod, recipient_name: Option<String>, recipient_address: Option<String>, circumstances: Option<String>, euthanasia_by: Option<String>, euthanasia_method: Option<String> }`
- `struct Story { breed: Option<String>, age_text: Option<String>, birth_date: Option<NaiveDate>, color_markings: Option<String>, weight_lbs: Option<f32>, altered: YesNoUnknown, good_with_dogs: YesNoUnknown, good_with_cats: YesNoUnknown, good_with_kids: YesNoUnknown, house_trained: YesNoUnknown, energy: Option<Energy>, personality: Option<String>, special_needs: Option<String>, adoption_fee: Option<u32> }`
- `struct Animal { id: AnimalId, name: String, species: Species, sex: Sex, received_at: NaiveDateTime, source: Option<Source>, out_of_state: bool, health_certificate: Option<String>, identification: Identification, health: Vec<HealthEntry>, story: Story, photo: Option<String>, status: Status, status_changed_at: NaiveDateTime, disposition: Option<Disposition> }`
- `struct Settings { rescue_name: String, phone: String, email: String, website: String, hashtags: Vec<String> }` + `impl Default` (`"Your Rescue Name"`, `"(402) 555-0100"`, `"adopt@example.org"`, `""`, `["#AdoptDontShop", "#NebraskaRescue"]`)
- All derive `Clone, Debug, PartialEq, Serialize, Deserialize`.
- `rescue_core::store_data`: `struct StoreData { version: u32, animals: Vec<Animal>, settings: Settings }`, `const SCHEMA_VERSION: u32 = 1`, `fn to_json(&self) -> String`, `fn from_json(s: &str) -> Result<StoreData, StoreError>` where `enum StoreError { Corrupt(String), WrongVersion(u32) }`.

- [ ] **Step 1:** Install toolchain: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y`, `rustup target add wasm32-unknown-unknown`, install Trunk (prebuilt release binary to `~/.cargo/bin`). Verify `cargo --version && trunk --version`.
- [ ] **Step 2: Failing tests** in `store_data.rs`: `round_trip_preserves_animals` (build `StoreData` with one animal incl. microchip + one health entry; `from_json(to_json())` equals original); `corrupt_json_is_error` (`from_json("{not json")` → `Err(Corrupt(_))`); `future_version_is_error` (version 99 → `Err(WrongVersion(99))`); `identification_has_any` (empty → false, tag only → true).
- [ ] **Step 3:** Run `cargo test -p rescue-core` → FAIL (types missing).
- [ ] **Step 4:** Implement model + store_data.
- [ ] **Step 5:** `cargo test -p rescue-core` → PASS. Commit `feat(core): data model and storage schema`.

### Task 2: Compliance checklist

**Files:** Create `crates/rescue-core/src/compliance.rs`

**Interfaces — Produces:**
- `enum ItemState { Done, Open { due: NaiveDateTime }, Overdue { due: NaiveDateTime }, NotApplicable }`
- `struct ComplianceItem { key: &'static str, label: &'static str, section: &'static str, state: ItemState }`
- `fn check(animal: &Animal, now: NaiveDateTime) -> Vec<ComplianceItem>`

Rules (due = `received_at + 48h` for intake/health; `disposition.date (00:00) + 48h`, or `status_changed_at + 48h` if no date, for disposition):

| key | label | section | Done when |
|---|---|---|---|
| `source` | Source name, address & ID | 23 NAC 18-010.04A(3) | Person: all three fields non-blank; Agency: all three non-blank |
| `identification` | Individual ID (tag, cage card, tattoo, or chip) | 23 NAC 18-008 / 010.04A(4) | `identification.has_any()` |
| `microchip_details` | Microchip manufacturer & location | 23 NAC 18-008.07 | N/A if no chip; else both Some & non-blank |
| `health_certificate` | Health certificate (imported animals) | 23 NAC 18-010.04A(5) | N/A unless `out_of_state`; else non-blank |
| `health_record` | Health record started | 23 NAC 18-010.06 | `health` non-empty |
| `rabies` | Rabies vaccination recorded | 23 NAC 18-010.06A | any `RabiesVaccination` entry |
| `disposition` | Outcome record (date, recipient, method) | 23 NAC 18-010.05A | N/A unless status Outcome; Adopted/Transferred/ReturnedToOwner need date+name+address; Died/Euthanized need date+circumstances, Euthanized also by+method |

- [ ] **Step 1: Failing tests:** `fresh_intake_items_open_with_48h_due` (received 2026-11-20 10:00, now 11-20 12:00 → `source` is `Open{due: 11-22 10:00}`); `past_48h_is_overdue` (now 11-23 → `Overdue`); `complete_person_source_is_done`; `blank_dl_not_done`; `microchip_details_na_without_chip`; `health_cert_only_when_out_of_state`; `euthanized_requires_by_and_method`; `disposition_na_before_outcome`.
- [ ] **Step 2:** `cargo test -p rescue-core compliance` → FAIL.
- [ ] **Step 3:** Implement `check`.
- [ ] **Step 4:** PASS. Commit `feat(core): Nebraska compliance checklist`.

### Task 3: Social post text

**Files:** Create `crates/rescue-core/src/posts.rs`

**Interfaces — Produces:**
- `enum Tone { Upbeat, Heartfelt, Short }` with `ALL` and `label()`
- `struct Post { tone: Tone, text: String }`
- `fn generate_posts(animal: &Animal, settings: &Settings) -> Vec<Post>` (one per tone)
- `fn fact_lines(animal: &Animal) -> Vec<(&'static str, String)>` — label/value pairs for fact sheet & share image, skipping unknowns (e.g. `("Breed","Lab mix")`, `("Age","~2 years")`, `("Good with","dogs, kids")`, `("Spayed/neutered","Yes")`, `("Fee","$150")`).

Upbeat and Heartfelt end with a contact line built from settings (`"Contact {rescue_name}: {phone} · {email}"`, empty parts dropped) and the hashtags joined by spaces. Short is ≤280 chars incl. hashtags. Pronoun from sex: he/she/they.

- [ ] **Step 1: Failing tests:** `sparse_animal_has_no_none_or_double_spaces` (only name+species set: no post contains `"None"`, `"Unknown"`, `"  "`, or `"$"`); `includes_name_breed_and_fee_when_present`; `short_post_within_280`; `contact_line_drops_empty_website`; `female_uses_she`, `unknown_sex_uses_they`; `fact_lines_skip_unknown_good_with`.
- [ ] **Step 2:** FAIL. **Step 3:** Implement. **Step 4:** PASS. Commit `feat(core): post text and fact lines`.

### Task 4: Sample animals

**Files:** Create `crates/rescue-core/src/samples.rs`; add `web/assets/samples/` photos (generated or public-domain/CC0 dog & cat images, ≤150 KB each, credit in `assets/samples/CREDITS.md`)

**Interfaces — Produces:** `fn sample_animals(now: NaiveDateTime, photos: &[Option<String>; 3]) -> Vec<Animal>` (photo strings are relative asset URLs)

Three animals relative to `now`: **Biscuit** (dog, Available, complete records, microchipped); **Pepper** (cat, VetCheck, received 3 days ago, missing DL + rabies → overdue items); **Moose** (dog, Outcome/Adopted with complete disposition).

- [ ] **Step 1: Failing test** `samples_cover_three_states`: statuses are {Available, VetCheck, Outcome}; `check(pepper)` has ≥1 `Overdue`; `check(biscuit)` has none `Open`/`Overdue`.
- [ ] **Step 2–4:** FAIL → implement → PASS. Commit `feat(core): sample animals`.

### Task 5: Web shell — Trunk, router, store, roster, reset

**Files:** Create `crates/web/{Cargo.toml,index.html,Trunk.toml,style.css}`, `src/main.rs`, `src/app.rs` (hash router + layout: header, banner, footer), `src/store.rs`, `src/pages/{mod.rs,roster.rs}`, `src/clock.rs`

**Interfaces — Produces:**
- `clock::now() -> NaiveDateTime` (from `js_sys::Date`, local time)
- `trait Store { fn load(&self) -> Result<StoreData, StoreError>; fn save(&self, d: &StoreData) -> Result<(), String>; }`, `struct LocalStore` (key `rescue-demo-v1`)
- `AppState` provided via context: `data: RwSignal<StoreData>`, `notice: RwSignal<Option<String>>`, `fn persist(&self) -> Result<(), String>`, `fn upsert(&self, a: Animal) -> Result<(), String>`, `fn reset(&self)`
- `enum Route { Roster, New, Animal(AnimalId), Sheet(AnimalId), Posts(AnimalId), Settings, NotFound }`, `fn parse_hash(h: &str) -> Route`, `fn href(r: &Route) -> String`

Load: missing key → samples; `Err` → samples + notice "Saved demo data couldn't be read and was reset". Roster: card grid (photo/placeholder, name, species·breed, status badge), filter chips In care / Available / Outcomes / All, "+ New intake", "Reset demo" (confirm dialog).

- [ ] **Step 1: Failing tests** (host, in `app.rs` pure fn): `parse_hash` maps `"#/animal/abc/posts"` → `Posts("abc")`, `""` and `"#/"` → `Roster`, `"#/bogus"` → `NotFound`; `href(parse_hash(x))` round-trips.
- [ ] **Step 2–4:** FAIL → implement → PASS (`cargo test -p web`).
- [ ] **Step 5:** `trunk serve` in `crates/web`; open in browser (webapp-testing/Playwright): roster shows 3 sample cards; reset restores after deleting localStorage manually; corrupt localStorage value → notice shown.
- [ ] **Step 6:** Commit `feat(web): shell, router, store, roster`.

### Task 6: Intake form + photo

**Files:** Create `src/pages/intake.rs`, `src/components/{mod.rs,hint.rs,photo_input.rs,yes_no.rs}`, `src/photo.rs`

**Interfaces — Produces:**
- `photo::downscale(file: web_sys::File, max: u32, quality: f64) -> Result<String, String>` (async; `createImageBitmap` → canvas → `toDataURL("image/jpeg", q)`)
- `<Hint section="23 NAC 18-010.04A(3)" text=.../>` small "why we ask" disclosure
- `<PhotoInput value: RwSignal<Option<String>>/>` (`accept="image/*"`, `capture` unset so phones offer camera or library)
- `<YesNo value: RwSignal<YesNoUnknown> label/>`
- `fn validate_minimum(name: &str, species: Option<Species>, received: Option<NaiveDateTime>) -> Result<(), Vec<&'static str>>` in `intake.rs` (host-testable)

Sections in order: Animal · Identification · Source (Person/Agency toggle) · Health (add-row list) · Story · Photo. "Out of state?" checkbox reveals health certificate field. On submit: validate minimum → build `Animal` (uuid id, status `Intake`, `status_changed_at = received_at`) → `upsert` → navigate to profile; on save error show it and keep the form.

- [ ] **Step 1: Failing test** `validate_minimum_requires_three_fields` (empty name → error lists "Name"; all present → Ok).
- [ ] **Step 2–4:** FAIL → implement → PASS.
- [ ] **Step 5:** Browser check: complete an intake with a large JPEG; new animal appears on roster; stored photo data URL < 400 KB; a non-image file shows the error text from Review Focus 1.
- [ ] **Step 6:** Commit `feat(web): intake form with photo`.

### Task 7: Animal profile — edit, statuses, compliance, outcome

**Files:** Create `src/pages/profile.rs`, `src/components/{status_stepper.rs,compliance_list.rs,disposition_form.rs}`; refactor intake sections into reusable `src/components/animal_form.rs` used by both new and edit.

**Interfaces — Consumes:** `compliance::check`, `AppState::upsert`, `clock::now`. **Produces:** `<AnimalForm animal: Animal on_save: Callback<Animal>/>`.

Profile shows photo, details, status stepper (clickable pills; changing status sets `status_changed_at = now`; choosing Outcome reveals `DispositionForm` with method select; Euthanized shows by/method fields, Died/Euthanized show circumstances), compliance list (✅ Done, ⏳ "due in Xh", 🔴 "overdue by Xh", N/A hidden, each with section cite), Edit toggle, links to Fact sheet and Social posts.

- [ ] **Step 1:** Browser check on Pepper: two red overdue items; fill DL → item turns ✅ after save; set Outcome → Euthanized shows the extra fields; roster badge updates.
- [ ] **Step 2:** `cargo test --workspace` still PASS. Commit `feat(web): profile, statuses, compliance`.

### Task 8: Fact sheet

**Files:** Create `src/pages/fact_sheet.rs`; add `@media print` rules to `style.css`

Layout: rescue name header, large photo (or species placeholder SVG), name, `fact_lines` as a two-column list, personality paragraph, special needs, fee, contact block, "Print" button (`window.print()`). Print CSS hides header/banner/footer/buttons; Letter, one page.

- [ ] **Step 1:** Browser check: Biscuit sheet renders; print-to-PDF via Playwright `page.pdf({format:'Letter'})` produces exactly 1 page; sparse animal shows no empty rows.
- [ ] **Step 2:** Commit `feat(web): printable fact sheet`.

### Task 9: Social posts + share image

**Files:** Create `src/pages/posts.rs`, `src/share_image.rs`

**Interfaces — Produces:** `share_image::render(canvas: &HtmlCanvasElement, animal: &Animal, settings: &Settings) -> Result<(), String>` — 1080×1080: photo cover-cropped top 70%, bottom band with name (large), up to 3 `fact_lines`, rescue name; placeholder background when no photo.

Page: one card per `generate_posts` tone with textarea (editable) + "Copy" (Clipboard API; fallback select-all + message), canvas preview, "Download image" (`toDataURL("image/png")` → `<a download="{name}-adopt-me.png">`).

- [ ] **Step 1:** Browser check: three posts shown; Copy puts text on clipboard; download produces a 1080×1080 PNG; animal without photo still renders.
- [ ] **Step 2:** Commit `feat(web): social posts and share image`.

### Task 10: Settings

**Files:** Create `src/pages/settings.rs`

Form for `Settings` fields (hashtags as space-separated text); Save persists; link in header.

- [ ] **Step 1:** Browser check: change rescue name → appears on fact sheet, posts, share image.
- [ ] **Step 2:** Commit `feat(web): rescue settings`.

### Task 11: Polish, deploy, verify live

**Files:** `style.css` (warm palette, large tap targets, 16px gutters, single column <640px), `.github/workflows/pages.yml`, `README.md` (what it is, run locally with `trunk serve`, data-stays-local note, regulation source)

Workflow: on push to `main` → checkout → `dtolnay/rust-toolchain@stable` with `wasm32-unknown-unknown` → install Trunk (pinned release binary) → `cargo test -p rescue-core` → `trunk build --release --public-url /rescue-intake-demo/` in `crates/web` → `actions/upload-pages-artifact` (`crates/web/dist`) → `actions/deploy-pages`.

- [ ] **Step 1:** Playwright at 375×812: no horizontal scroll on roster, intake, profile, posts; screenshots saved to scratchpad.
- [ ] **Step 2:** `gh repo create AnsibleBBridge/rescue-intake-demo --public --source . --push`; enable Pages with source GitHub Actions (`gh api -X POST repos/AnsibleBBridge/rescue-intake-demo/pages -f build_type=workflow`).
- [ ] **Step 3:** Watch the run (`gh run watch`); open `https://ansiblebbridge.github.io/rescue-intake-demo/` in Playwright: roster loads, intake round-trip works, refresh on `#/animal/<id>` works.
- [ ] **Step 4:** Commit/push any fixes; report the live URL.
