mod affordability;
mod amortization;
mod compare;
mod payment;
mod refinance;

pub use affordability::AffordabilityScreen;
pub use amortization::AmortizationScreen;
pub use compare::CompareScreen;
pub use payment::PaymentScreen;
pub use refinance::RefinanceScreen;

use mortgage_calc::PaymentFrequency;

pub(crate) fn parse_frequency(s: &str) -> PaymentFrequency {
    match s {
        "biweekly" => PaymentFrequency::BiWeekly,
        "weekly" => PaymentFrequency::Weekly,
        _ => PaymentFrequency::Monthly,
    }
}
