// ---------------------------------------------------------------------------
// Multi-Factor Authentication types
// ---------------------------------------------------------------------------

/// Represents a single MFA requirement.
#[derive(Debug, Clone)]
pub enum MFARequirement {
    /// A specific factor ID that must be completed.
    Factor(String),
    /// One of the listed factor IDs must be completed.
    OneOf(Vec<String>),
    /// All of the listed factor IDs must be completed (in any order).
    AllOfInAnyOrder(Vec<String>),
}

/// A list of MFA requirements that must be satisfied.
pub type MFARequirementList = Vec<MFARequirement>;

/// Result of getting factors setup for a user.
#[derive(Debug, Clone)]
pub struct GetFactorsSetupForUserOkResult {
    pub factor_ids: Vec<String>,
}

/// Result of getting required secondary factors for a user.
#[derive(Debug, Clone)]
pub struct GetRequiredSecondaryFactorsOkResult {
    pub factor_ids: Vec<String>,
}

/// Result of marking a factor as complete in a session.
#[derive(Debug, Clone)]
pub struct MarkFactorAsCompleteOkResult;
