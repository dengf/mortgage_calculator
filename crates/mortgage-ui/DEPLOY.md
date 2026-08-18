# Deploying mortgage-ui

What's genuinely deploy-ready right now, and what still needs *you*
specifically (accounts, payment, credentials — nothing an agent can do on
your behalf).

## Web — ready to deploy today

`dx bundle --platform web --release` produces a static site at
`crates/mortgage-ui/dist/public/` (verified working locally: all 5 screens,
IndexedDB-backed scenario storage). It's just static files — host it
anywhere: GitHub Pages, Cloudflare Pages, Netlify, Vercel, S3, etc.

**The `deploy-web.yml` GitHub Actions workflow builds and deploys this to
GitHub Pages automatically on every push to `main`.** Your one-time step:

- [ ] **Enable GitHub Pages**: repo Settings → Pages → Source → set to
      "GitHub Actions" (github.com/dengf/mortgage_calculator/settings/pages)

That's the only step. After that, pushes to `main` auto-deploy.

## Android — signed release artifacts built, Play Store needs your account

A real release-signed APK and AAB (the format Play Store requires) already
exist at `crates/mortgage-ui/dist/mortgage-ui-release.{apk,aab}` — signed
with a real certificate (not the debug key), verified by installing and
launching on an emulator.

**The signing keystore** is at `crates/mortgage-ui/android-keystore/release.keystore`
— gitignored, never committed. Password is currently the placeholder
`mortgagecalc123` used to generate it.

- [ ] **Back up the keystore file somewhere safe** (password manager, secure
      cloud storage). If you lose it, you can never update this app under
      the same identity again — Play Store ties an app's identity to its
      signing key permanently.
- [ ] **Change the keystore password** from the placeholder before any real
      use:
      ```bash
      keytool -storepasswd -keystore crates/mortgage-ui/android-keystore/release.keystore
      keytool -keypasswd -keystore crates/mortgage-ui/android-keystore/release.keystore -alias mortgage-calculator
      ```
- [ ] **To let CI build signed releases too** (`build-android.yml` is
      already wired for this — it just needs these 4 secrets in repo
      Settings → Secrets and variables → Actions):
      ```bash
      base64 -i crates/mortgage-ui/android-keystore/release.keystore | pbcopy
      # paste as secret: ANDROID_KEYSTORE_BASE64
      ```
      Plus `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS` (`mortgage-calculator`),
      `ANDROID_KEY_PASSWORD` as secrets too.
- [ ] **To actually publish to Google Play**, you need a
      [Google Play Console account](https://play.google.com/console/signup)
      ($25 one-time fee, your Google account + payment — I can't do this
      part). Once you have one: create an app listing, upload
      `mortgage-ui-release.aab`, fill in the store listing (screenshots,
      description, privacy policy URL, content rating questionnaire), and
      submit for review.
- [ ] For sideloading/testing right now without any of the above: `adb
      install crates/mortgage-ui/dist/mortgage-ui-release.apk` on any
      Android device with USB debugging enabled, or just send someone the
      APK file directly.

## iOS — builds for Simulator only; App Store needs your Apple Developer account

`dx bundle --platform ios --release` produces `MortgageUi.app`, built for
**arm64** (real device architecture — correct for actual deployment, but
not launchable on this Intel Mac's Simulator, which needs an x86_64
build). The app's functionality was verified separately and thoroughly via
`dx serve --ios` against the booted Simulator (all 5 screens, calculations
matching web/desktop/Android exactly). This bundle is **not** signed for a
real device or the App Store; iOS requires code signing tied to an Apple
Developer account for anything beyond a local Simulator run, and I have no
way to create or pay for that account on your behalf.

- [ ] **Enroll in the [Apple Developer Program](https://developer.apple.com/programs/enroll/)**
      ($99/year, your Apple ID + payment).
- [ ] Once enrolled, the easiest path to a real device build is opening
      `target/dx/mortgage-ui/release/ios/app` (or re-running `dx bundle
      --platform ios` after enrolling) in Xcode and using its automatic
      signing (Xcode → Signing & Capabilities → select your team) — this
      needs to happen on your machine since it's tied to your Apple ID
      session in Xcode, not something scriptable from here.
- [ ] For TestFlight/App Store distribution specifically, you'll also need
      to create the app's record in
      [App Store Connect](https://appstoreconnect.apple.com/) (same Apple
      Developer account) before you can upload a build.
- [ ] The `build-ios.yml` GitHub Actions workflow has a comment block
      showing what it'd need (`APPLE_TEAM_ID` secret + an imported signing
      certificate/provisioning profile) once you're enrolled — I didn't
      wire this up since there's nothing to test it against yet.

## Before any store submission (both platforms)

- [ ] **Change the bundle identifier** in `crates/mortgage-ui/Dioxus.toml`
      (`[bundle] identifier = "io.github.dengf.mortgagecalculator"`) to
      something you actually control — ideally a domain you own
      (`com.yourdomain.mortgagecalculator`), since `io.github.*` works but
      isn't really "yours" the way a domain-backed identifier is. This is a
      one-line config change, not something requiring your credentials —
      happy to do this whenever you pick a value.
- [ ] App icon: currently using Dioxus's default. Both stores require a
      real icon in specific sizes before submission.
- [ ] Privacy policy: both app stores require a privacy-policy URL, even
      for an app doing nothing but local calculations. (This one's
      genuinely simple to justify — the app only stores data locally on
      the device.)

## What's fully automated already (GitHub Actions, `.github/workflows/`)

| Workflow | Runs on | What it does |
|---|---|---|
| `ci.yml` | push/PR to `main` | `cargo test --workspace`, plus builds the original `www/` React frontend |
| `deploy-web.yml` | push to `main` | Builds and deploys `mortgage-ui`'s web target to GitHub Pages |
| `build-android.yml` | push to `main` | Always builds a debug-signed APK (artifact); builds release-signed APK+AAB too if the 4 keystore secrets above are set |
| `build-ios.yml` | push to `main` | Builds the unsigned Simulator `.app` (artifact) |

None of these require anything from you except the GitHub Pages toggle and
(optionally) the Android signing secrets — they'll just run and produce
downloadable build artifacts on every push otherwise.
