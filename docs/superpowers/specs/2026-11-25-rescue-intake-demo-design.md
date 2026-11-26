# Rescue Intake Demo — Design

*2026-11-25 · status: draft for approval*

## Intent

A clickable demo for a friend who runs a small dog/cat rescue in Nebraska. She should be
able to open a link, fill out an intake form for an animal (including a photo), submit it,
and walk through the workflow to the payoff: a printable fact sheet and ready-to-share
social media posts that help get the animal adopted.

**Why:** show her what's possible so her reactions shape the real app. This is the first
step of a work in progress; the real app (Rust server on Proxmox, shared data, logins,
comments) comes later and should reuse as much of this as possible.

## Goals

- Static site on GitHub Pages; works on her phone and on a computer.
- Intake form capturing the records Nebraska requires of licensed animal rescues (below),
  plus the "adoption-story" info that feeds fact sheets and posts.
- Photo upload (from phone camera or file) shown on the profile, fact sheet, and post image.
- Workflow statuses she can move an animal through.
- Fact sheet: printable one-pager (browser print → PDF).
- Social posts: generated text variants with copy buttons, plus a downloadable square
  share image (photo + name + key facts).
- Pre-loaded sample animals and a "Reset demo" button.

## Non-goals (for this demo)

- No server, no shared data, no real login, no comments/markup (deferred — see Later).
- No direct posting to Facebook/Instagram; copy/download only.
- No AI-written posts; posts come from templates filled with the animal's details.
- No adopter applications, fosters, donations, or vet-scheduling.

## Anti-goals

- **Looking like legal advice.** Every page footer says: demo only; confirm requirements with
  the Nebraska Department of Agriculture.
- **Real personal data leaving her browser.** Nothing is sent anywhere; no analytics,
  no third-party scripts beyond what the build needs.
- **A blank, confusing first impression.** It must open onto a populated roster.

## Constraints

- **Rust, compiled to WebAssembly, using Leptos (client-side rendering)** — chosen by Bryan
  so the data model and rendering logic carry into the future server app.
- **GitHub Pages hosting**, built and deployed by a GitHub Action on push.
  ⚠️ On a free GitHub org, Pages requires a **public repo**. Acceptable because the code
  holds only fake sample data; her entries never leave her browser.
- **Storage is her browser** (localStorage). Photos are downscaled in-browser before saving
  (~1200px longest side, JPEG) so a handful of animals fit within browser storage limits.
- Site is served from a sub-path (`/<repo-name>/`), so routing and asset links must respect it.

## Nebraska requirements the form covers

Source: Nebraska Dept. of Agriculture, *Commercial Dog & Cat Operator Inspection
Regulations*, 23 NAC 18, effective 2016-02-17 (NDA's own PDF). Section refs shown in the UI
as small "why we ask" hints.

**Acquisition record — 23 NAC 18-010.04A** (complete within 48 h of intake):
- Date received
- Source: person's name, complete mailing address, **driver's license number** — OR, for a
  government source, the agency name, official title, and mailing address
- Individual identification (see below)
- For animals imported into Nebraska: health certificate (APHIS 7001 or state form) — a
  yes/no + certificate number field, shown when "came from out of state" is checked

**Identification — 23 NAC 18-008:**
- At least one method: ID tag, cage card, tattoo, or microchip; record all existing IDs
- If microchip: chip number, **manufacturer**, **approximate location** on the body
- Litters with their mother may share the mother's ID until separated
- IDs kept ≥3 years after disposition (008.07; also 010 general 3-year retention)

**Health record — 23 NAC 18-010.06** (within 48 h; travels with the animal at adoption):
- Vaccinations incl. rabies certificate; treatments and medications
- Medical procedures performed (e.g. spay/neuter)
- Reasons/conditions for treatment and results

**Disposition record — 23 NAC 18-010.05A** (within 48 h of leaving):
- Date; name and complete mailing address of recipient; animal's ID; method
  (adopted, transferred, returned to owner, died, euthanized)
- If died/euthanized: circumstances; if euthanized, who performed it and method

Shown on the profile as an "Outcome" step when status becomes Adopted/Transferred/etc.,
so the demo shows the full legal lifecycle, not just intake.

A small **"Compliance checklist"** on each profile shows which required items are filled
and flags anything past its 48-hour window.

## Adoption-story fields (for fact sheets and posts)

Name, species (dog/cat), breed/mix, sex, age or birth date, color/markings, weight,
spayed/neutered, good with dogs / cats / kids (yes / no / unknown), house-trained,
energy level, personality blurb, special needs, adoption fee, photo.

## Screens

```
Roster (home) ─▶ New intake (form) ─▶ Animal profile ─┬─▶ Fact sheet (print)
   cards w/ photo + status badge       status stepper  └─▶ Social posts
                                       compliance list      (text variants + share image)
```

1. **Roster** — photo cards, status badges, filter by status, "+ New intake", "Reset demo".
2. **Intake form** — sections: Animal · Identification · Source · Health · Story · Photo.
   Required-by-law fields marked; submit validates and goes to the new profile.
3. **Animal profile** — all details, editable; status stepper
   *Intake → Vet check → Available → Adopted/Outcome*; compliance checklist;
   buttons to Fact sheet and Social posts.
4. **Fact sheet** — clean one-page layout (photo, name, key facts, personality, fee,
   rescue contact). Print stylesheet hides app chrome.
   **Reference (Bryan, 2026-11-25, Pinterest/Canva "Pet Profile" template):** keep this
   structure in our own styling (not a copy of the template's art): title band
   ("Meet <Name>" / Pet Profile) → **Overview** card (Breed, Sex, Age, Weight, Color, Size
   potential) beside a **large circular photo** with the name on a ribbon/banner below it →
   **Background** card (personality paragraph; good-with notes) → full-width **footer band**
   "Interested in adopting? Call/Email …" from Settings. Soft pastel palette, rounded cards.
   **No "Fun Facts" section** (crossed out on the reference). Adds `Story.size_potential:
   Option<String>` (e.g. "about 10 lbs grown").
5. **Social posts** — 2–3 tone variants (e.g. upbeat, heartfelt, short), each with a copy
   button and hashtags; a square share image drawn on a canvas, downloadable as PNG.

## How it's built (decided)

- Leptos CSR app built with `trunk`; GitHub Action builds and publishes to Pages.
- Plain Rust data model (`Animal`, `Source`, `Identification`, `HealthEntry`, `Disposition`,
  `Status`) kept free of browser/UI code so the future server app can reuse it.
- Post-text generation and compliance checks are pure Rust functions with unit tests.
- Storage behind a small `Store` trait (localStorage now; server API later).

## Calls made after builder review (Claude's calls — correct if wrong)

- **Save and flag, never block.** Submit requires only name, species, and date received.
  Legally required fields that are missing show as open items on the compliance checklist
  with a countdown to the 48-hour deadline; overdue items turn red. Model fields are
  `Option` so incomplete records are a normal stored state (the real app needs this too).
- **Litters: hint only.** The 18-008 litter rule appears as a hint on the ID section; no
  mother/litter modelling in the demo.
- **Rescue contact: placeholder + small Settings panel.** Defaults to "Your Rescue Name" and
  a placeholder email/phone; a Settings panel (rescue name, phone, email, website, default
  hashtags) feeds the fact sheet, posts, and share image. Her real details never go in the repo.
- **Throwaway data.** A persistent banner says "Demo — entries stay in this browser only";
  Reset wipes everything and restores the samples. No export/import in the demo.
- **Statuses:** `Intake → Vet check → Medical hold (optional) → Available → Adoption pending →
  Outcome`. She can move freely between any statuses (it's a demo). Outcome carries a method
  (Adopted, Transferred, Returned to owner, Died, Euthanized); choosing it opens the
  disposition fields. Roster filter groups: In care / Available / Outcomes.
- **Repo:** `AnsibleBBridge/rescue-intake-demo` (public, for free Pages); Trunk `--public-url`
  set from the repo name so the base path is one config value.

## Left to the builder

Visual styling (warm, friendly, legible on a phone), exact post template wording, sample
animal content, routing approach, and the share-image layout.

## Later (not this demo)

Rust server on Proxmox via Cloudflare Tunnel or Tailscale Funnel; shared data + real login;
in-app feedback notes committed to the repo; drawing/markup.

**Exports (Bryan, 2026-11-25 — "eventually"):** spreadsheet export (CSV/XLSX of animals and
their acquisition/disposition records) and PDF reports / printable records (e.g. a per-animal
record covering 010.04A, 010.06, 010.05A — the paperwork an NDA inspector would ask for — and
roster/period reports). Design implication for now: keep all record data structured in
`rescue-core` (no free-text blobs) so export functions are pure Rust over `StoreData`.
