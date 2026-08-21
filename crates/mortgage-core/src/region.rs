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
