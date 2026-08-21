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
        let code = s.rsplit(['-', '_']).next().unwrap_or(s);
        if code.eq_ignore_ascii_case("SG") {
            Region::SG
        } else {
            Region::US
        }
    }
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

    #[test]
    fn as_str_round_trips_through_parse() {
        assert_eq!(Region::parse(Region::US.as_str()), Region::US);
        assert_eq!(Region::parse(Region::SG.as_str()), Region::SG);
    }
}
