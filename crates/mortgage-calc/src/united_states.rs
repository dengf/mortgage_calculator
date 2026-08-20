//! United States-specific calculations: ZIP-code property tax estimation,
//! the private mortgage insurance (PMI) trigger below 20% down, and a
//! federal tax-deductibility simulator for mortgage interest.

use mortgage_core::round_currency;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Maps a ZIP code's first three digits to a state via published USPS ZIP3
/// prefix ranges. This is a coarse regional estimate — some ZIP3 prefixes
/// are unassigned or span military/territory codes not covered here — not
/// a precise county-level lookup.
pub fn zip3_to_state(zip: &str) -> Option<&'static str> {
    let prefix: u32 = zip.get(0..3)?.parse().ok()?;
    ZIP3_RANGES
        .iter()
        .find(|(lo, hi, _)| prefix >= *lo && prefix <= *hi)
        .map(|(_, _, state)| *state)
}

#[rustfmt::skip]
const ZIP3_RANGES: &[(u32, u32, &str)] = &[
    (6, 9, "PR"), (10, 27, "MA"), (28, 29, "RI"), (30, 38, "NH"), (39, 49, "ME"),
    (50, 59, "VT"), (60, 69, "CT"), (70, 89, "NJ"), (100, 149, "NY"), (150, 196, "PA"),
    (197, 199, "DE"), (200, 205, "DC"), (206, 219, "MD"), (220, 246, "VA"), (247, 268, "WV"),
    (270, 289, "NC"), (290, 299, "SC"), (300, 319, "GA"), (320, 349, "FL"), (350, 369, "AL"),
    (370, 385, "TN"), (386, 397, "MS"), (398, 399, "GA"), (400, 427, "KY"), (430, 459, "OH"),
    (460, 479, "IN"), (480, 499, "MI"), (500, 528, "IA"), (530, 549, "WI"), (550, 567, "MN"),
    (570, 577, "SD"), (580, 588, "ND"), (590, 599, "MT"), (600, 629, "IL"), (630, 658, "MO"),
    (660, 679, "KS"), (680, 693, "NE"), (700, 714, "LA"), (716, 729, "AR"), (730, 749, "OK"),
    (750, 799, "TX"), (800, 816, "CO"), (820, 831, "WY"), (832, 838, "ID"), (840, 847, "UT"),
    (850, 865, "AZ"), (870, 884, "NM"), (889, 898, "NV"), (900, 961, "CA"), (967, 968, "HI"),
    (970, 979, "OR"), (980, 994, "WA"), (995, 999, "AK"),
];

/// Average effective property tax rate (annual, as a fraction of home
/// value) by state, e.g. `0.0027` for Hawaii's ~0.27%. Source: published
/// state-level effective rate rankings; actual rates vary by county/city.
pub fn property_tax_rate_for_state(state: &str) -> Option<Decimal> {
    STATE_PROPERTY_TAX_RATES
        .iter()
        .find(|(code, _)| *code == state)
        .map(|(_, rate)| *rate)
}

/// Convenience: ZIP code straight to an estimated annual property tax rate.
pub fn estimate_property_tax_rate(zip: &str) -> Option<Decimal> {
    property_tax_rate_for_state(zip3_to_state(zip)?)
}

#[rustfmt::skip]
const STATE_PROPERTY_TAX_RATES: &[(&str, Decimal)] = &[
    ("HI", dec!(0.0027)), ("AL", dec!(0.0038)), ("NV", dec!(0.0047)), ("AZ", dec!(0.0048)),
    ("CO", dec!(0.0048)), ("SC", dec!(0.0048)), ("ID", dec!(0.0049)), ("DE", dec!(0.0050)),
    ("TN", dec!(0.0050)), ("UT", dec!(0.0052)), ("WV", dec!(0.0053)), ("LA", dec!(0.0055)),
    ("AR", dec!(0.0055)), ("WY", dec!(0.0057)), ("DC", dec!(0.0058)), ("NC", dec!(0.0066)),
    ("NM", dec!(0.0070)), ("CA", dec!(0.0070)), ("MT", dec!(0.0072)), ("MS", dec!(0.0072)),
    ("VA", dec!(0.0073)), ("IN", dec!(0.0074)), ("KY", dec!(0.0075)), ("FL", dec!(0.0076)),
    ("GA", dec!(0.0077)), ("OK", dec!(0.0080)), ("OR", dec!(0.0081)), ("WA", dec!(0.0081)),
    ("MO", dec!(0.0085)), ("MD", dec!(0.0097)), ("ND", dec!(0.0099)), ("MN", dec!(0.0102)),
    ("ME", dec!(0.0102)), ("SD", dec!(0.0106)), ("MA", dec!(0.0107)), ("AK", dec!(0.0111)),
    ("RI", dec!(0.0121)), ("MI", dec!(0.0125)), ("KS", dec!(0.0129)), ("PA", dec!(0.0130)),
    ("OH", dec!(0.0131)), ("IA", dec!(0.0139)), ("WI", dec!(0.0142)), ("TX", dec!(0.0149)),
    ("NE", dec!(0.0149)), ("NY", dec!(0.0155)), ("VT", dec!(0.0159)), ("NH", dec!(0.0166)),
    ("CT", dec!(0.0181)), ("IL", dec!(0.0201)), ("NJ", dec!(0.0211)),
];

/// Conventional loans require PMI once the down payment drops below this
/// share of the home price.
pub const PMI_DOWN_PAYMENT_THRESHOLD: Decimal = dec!(0.20);

/// Representative annual PMI rate used when the user doesn't override it —
/// the middle of the commonly-cited 0.5%-1.5%-of-loan-amount range.
pub const DEFAULT_PMI_ANNUAL_RATE: Decimal = dec!(0.0075);

pub fn requires_pmi(down_payment_percent: Decimal) -> bool {
    down_payment_percent < PMI_DOWN_PAYMENT_THRESHOLD
}

/// Monthly PMI premium: an annual rate applied to the loan amount, billed monthly.
pub fn monthly_pmi(loan_amount: Decimal, annual_pmi_rate: Decimal) -> Decimal {
    round_currency(loan_amount.max(Decimal::ZERO) * annual_pmi_rate / dec!(12))
}

/// Monthly tax savings from deducting mortgage interest at the borrower's
/// marginal federal rate — a simplified "itemizing helps this much" figure,
/// not a full tax return simulation.
pub fn monthly_tax_savings(deductible_monthly_interest: Decimal, marginal_tax_rate: Decimal) -> Decimal {
    round_currency(deductible_monthly_interest.max(Decimal::ZERO) * marginal_tax_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zip3_resolves_known_prefixes() {
        assert_eq!(zip3_to_state("90210"), Some("CA"));
        assert_eq!(zip3_to_state("10001"), Some("NY"));
        assert_eq!(zip3_to_state("96815"), Some("HI"));
        assert_eq!(zip3_to_state("07030"), Some("NJ"));
    }

    #[test]
    fn zip3_rejects_malformed_input() {
        assert_eq!(zip3_to_state("ab"), None);
        assert_eq!(zip3_to_state(""), None);
    }

    #[test]
    fn hawaii_and_new_jersey_bracket_the_range_as_expected() {
        // These are the two examples the feature request itself cites.
        let hi = property_tax_rate_for_state("HI").unwrap();
        let nj = property_tax_rate_for_state("NJ").unwrap();
        assert!(hi < dec!(0.005));
        assert!(nj > dec!(0.02));
    }

    #[test]
    fn every_state_and_dc_has_a_rate() {
        assert_eq!(STATE_PROPERTY_TAX_RATES.len(), 51);
    }

    #[test]
    fn estimate_property_tax_rate_chains_zip_to_state() {
        assert_eq!(estimate_property_tax_rate("90210"), property_tax_rate_for_state("CA"));
    }

    #[test]
    fn pmi_triggers_below_twenty_percent_down() {
        assert!(requires_pmi(dec!(0.19)));
        assert!(!requires_pmi(dec!(0.20)));
        assert!(!requires_pmi(dec!(0.25)));
    }

    #[test]
    fn monthly_pmi_is_annual_rate_over_twelve() {
        let pmi = monthly_pmi(dec!(400_000), dec!(0.0075));
        assert_eq!(pmi, dec!(250.00));
    }

    #[test]
    fn tax_savings_scales_with_bracket() {
        let savings = monthly_tax_savings(dec!(2000), dec!(0.22));
        assert_eq!(savings, dec!(440.00));
    }
}
