mod correction;
mod omission;

pub use correction::{Correction, PersonCorrection};
pub use omission::{Omission, OmissionCategory, OmissionId, OmissionPlaceholders, OmissionType};

#[cfg(test)]
pub use omission::tests::sample_omission;
