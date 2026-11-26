# Rescue Intake Demo

A small demo for a Nebraska animal rescue: log an animal at intake, see its profile with a compliance checklist, then print an adoption fact sheet and copy ready-made social posts plus a share image.

**Live:** https://ansiblebbridge.github.io/rescue-intake-demo/

**Your data stays in your browser.** Everything is saved in this browser's local storage. Nothing is sent to a server, and clearing site data erases it.

**Demo only, not legal advice.** The checklist follows the Nebraska Department of Agriculture's Commercial Dog & Cat Operator Inspection Regulations (23 NAC 18, effective 2016-02-17): https://nda.nebraska.gov/sites/default/files/regulations/animal/CommercialDogCatOperatorInspRegulations.pdf. Confirm current requirements with the Department.

## Run it locally

```sh
cd crates/web
trunk serve
```

## Tests

```sh
cargo test --workspace      # unit tests
```

Browser (Playwright) tests: see [e2e/README.md](e2e/README.md).

## Built with

Rust, Leptos (client-side), Trunk, deployed on GitHub Pages.
