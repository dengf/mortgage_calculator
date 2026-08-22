use serde::{Deserialize, Serialize};

/// The regional ruleset a calculation should use: which regulatory limits
/// apply, which upfront costs exist, and how the UI should label things.
///
/// New regions are added here and then wired into each calculator/UI layer
/// that needs region-specific behavior; this type is just the shared tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Region {
    /// United States: PITI/escrow, PMI below 20% down, ZIP-code property tax.
    #[default]
    US,
    /// Singapore: CPF OA, MAS TDSR/MSR limits, BSD/ABSD stamp duty.
    SG,
}

impl Region {
    pub fn as_str(self) -> &'static str {
        match self {
            Region::US => "US",
            Region::SG => "SG",
        }
    }

    /// Parses a region code, matched case-insensitively (`"us"`, `"US"`,
    /// `"en-US"`'s trailing subtag, etc. all resolve via their last `-`
    /// segment). Unrecognized input falls back to [`Region::US`].
    pub fn parse(s: &str) -> Region {
        Region::try_parse(s).unwrap_or_default()
    }

    /// Like [`Region::parse`], but distinguishes "this names a region we
    /// support" from "this names nothing".
    ///
    /// [`Region::detect`] needs that distinction: falling back to `US` is
    /// right as a final answer and wrong as an intermediate one, because a
    /// signal that says nothing must let the next signal speak. Reading
    /// `en-GB` as `US` would end the search at the first language tag.
    pub fn try_parse(s: &str) -> Option<Region> {
        let code = s.rsplit(['-', '_']).next().unwrap_or(s);
        if code.eq_ignore_ascii_case("SG") {
            Some(Region::SG)
        } else if code.eq_ignore_ascii_case("US") {
            Some(Region::US)
        } else {
            None
        }
    }

    /// Maps an IANA time zone to the region whose rules apply there.
    ///
    /// `None` means "this zone is not one we have rules for", never "US" —
    /// see [`Region::try_parse`] for why that distinction matters. Only
    /// zones we actually claim appear here; `America/Toronto` is not a US
    /// mortgage market and is not listed as one.
    ///
    /// `Singapore` is the pre-2011 alias for `Asia/Singapore`, still emitted
    /// by some older Android and embedded platforms.
    pub fn from_time_zone(tz: &str) -> Option<Region> {
        match tz {
            "Asia/Singapore" | "Singapore" => Some(Region::SG),
            _ => None,
        }
    }

    /// Picks a starting region from whatever the host platform can observe,
    /// strongest signal first.
    ///
    /// Ranking time zone above language is the whole point. A language tag
    /// says what someone wants to read, not where they are: Chrome on a
    /// Singapore device typically reports `en-GB` or `en-US` with no region
    /// subtag at all, so a locale-only guess sent the overwhelmingly common
    /// case — a Singapore resident on default browser settings — to the US
    /// ruleset. That is not a cosmetic miss. The US model has no TDSR, no
    /// MSR, no LTV step-down and no ABSD in it, so the answer is wrong by
    /// six figures and looks entirely plausible.
    ///
    /// The time zone comes from the OS clock, which a resident sets once and
    /// correctly, and on every platform we target it is already local — no
    /// lookup service, no IP address leaving the device.
    pub fn detect(signals: RegionSignals<'_>) -> Region {
        signals
            .chosen
            .and_then(Region::try_parse)
            .or_else(|| signals.time_zone.and_then(Region::from_time_zone))
            .or_else(|| {
                signals
                    .locales
                    .iter()
                    .find_map(|tag| Region::try_parse(tag))
            })
            .unwrap_or_default()
    }
}

/// What a host platform can observe about which market its user is in.
///
/// Every field is a raw observation, not a decision: the ranking between
/// them is [`Region::detect`]'s job and lives here so the web and desktop
/// apps cannot disagree about it.
#[derive(Debug, Default, Clone)]
pub struct RegionSignals<'a> {
    /// A region the user chose outright — a stored preference, or a
    /// `?region=` link. Overrides every inferred signal, so a wrong guess
    /// only ever has to be corrected once.
    pub chosen: Option<&'a str>,
    /// The device's IANA time zone, e.g. `Asia/Singapore`.
    pub time_zone: Option<&'a str>,
    /// Preferred language tags, most preferred first.
    pub locales: &'a [&'a str],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_matches_bare_region_codes_case_insensitively() {
        assert_eq!(Region::parse("SG"), Region::SG);
        assert_eq!(Region::parse("sg"), Region::SG);
        assert_eq!(Region::parse("US"), Region::US);
        assert_eq!(Region::parse("us"), Region::US);
    }

    #[test]
    fn parse_matches_the_trailing_subtag_of_a_locale_string() {
        assert_eq!(Region::parse("en-SG"), Region::SG);
        assert_eq!(Region::parse("en_SG"), Region::SG);
        assert_eq!(Region::parse("zh-Hans-SG"), Region::SG);
        assert_eq!(Region::parse("en-US"), Region::US);
    }

    #[test]
    fn parse_falls_back_to_us_for_anything_unrecognized() {
        assert_eq!(Region::parse(""), Region::US);
        assert_eq!(Region::parse("fr-FR"), Region::US);
        assert_eq!(Region::parse("not a locale"), Region::US);
    }

    fn signals<'a>(
        chosen: Option<&'a str>,
        time_zone: Option<&'a str>,
        locales: &'a [&'a str],
    ) -> RegionSignals<'a> {
        RegionSignals {
            chosen,
            time_zone,
            locales,
        }
    }

    #[test]
    fn try_parse_separates_unknown_input_from_the_us_fallback() {
        assert_eq!(Region::try_parse("en-GB"), None);
        assert_eq!(Region::try_parse("fr-FR"), None);
        assert_eq!(Region::try_parse("en-US"), Some(Region::US));
        assert_eq!(Region::try_parse("en-SG"), Some(Region::SG));
    }

    #[test]
    fn a_singapore_time_zone_beats_a_language_that_names_no_region() {
        // The case that was getting this wrong in production: a Singapore
        // resident whose browser reports plain `en-GB`.
        assert_eq!(
            Region::detect(signals(None, Some("Asia/Singapore"), &["en-GB"])),
            Region::SG
        );
        assert_eq!(
            Region::detect(signals(None, Some("Singapore"), &["en-US", "zh-CN"])),
            Region::SG
        );
    }

    #[test]
    fn a_language_still_decides_when_the_time_zone_names_no_region() {
        assert_eq!(
            Region::detect(signals(None, Some("Europe/London"), &["en-SG"])),
            Region::SG
        );
        assert_eq!(
            Region::detect(signals(None, None, &["zh-Hans-SG"])),
            Region::SG
        );
    }

    #[test]
    fn an_explicit_choice_overrides_every_inferred_signal() {
        assert_eq!(
            Region::detect(signals(Some("US"), Some("Asia/Singapore"), &["en-SG"])),
            Region::US
        );
        assert_eq!(
            Region::detect(signals(Some("SG"), Some("America/New_York"), &["en-US"])),
            Region::SG
        );
    }

    #[test]
    fn a_meaningless_choice_does_not_silence_the_signals_beneath_it() {
        // A stale or hand-edited stored value must not pin everyone to the
        // default; it is absent input, not an answer.
        assert_eq!(
            Region::detect(signals(Some("XX"), Some("Asia/Singapore"), &[])),
            Region::SG
        );
    }

    #[test]
    fn detect_falls_back_to_us_when_nothing_is_observable() {
        assert_eq!(Region::detect(RegionSignals::default()), Region::US);
        assert_eq!(
            Region::detect(signals(None, Some("Europe/Paris"), &["fr-FR"])),
            Region::US
        );
    }

    #[test]
    fn as_str_round_trips_through_parse() {
        assert_eq!(Region::parse(Region::US.as_str()), Region::US);
        assert_eq!(Region::parse(Region::SG.as_str()), Region::SG);
    }
}
