use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FailOn {
    Error,
    Warn,
    Never,
}

impl Default for FailOn {
    fn default() -> Self {
        Self::Error
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Profile {
    /// Favor signal and stability. Default.
    Default,
    /// Fail on warnings.
    Strict,
    /// Never fail. Useful for advisory runs.
    Advisory,
}

impl Default for Profile {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FilterConfig {
    #[serde(default)]
    pub include_paths: Vec<String>,
    #[serde(default)]
    pub exclude_paths: Vec<String>,

    #[serde(default)]
    pub allow_codes: Vec<String>,
    #[serde(default)]
    pub suppress_codes: Vec<String>,
    #[serde(default)]
    pub deny_codes: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProvenanceConfig {
    #[serde(default)]
    pub record_rustc: bool,
    #[serde(default)]
    pub record_clippy: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LintdiffConfig {
    pub profile: Option<Profile>,
    pub fail_on: Option<FailOn>,
    pub max_findings: Option<usize>,
    pub max_annotations: Option<usize>,
    pub workspace_only: Option<bool>,

    #[serde(default)]
    pub filter: FilterConfig,

    #[serde(default)]
    pub provenance: ProvenanceConfig,
}

#[derive(Clone, Debug)]
pub struct EffectiveConfig {
    pub profile: Profile,
    pub fail_on: FailOn,
    pub max_findings: usize,
    pub max_annotations: usize,
    pub workspace_only: bool,
    pub filter: FilterConfig,
    pub provenance: ProvenanceConfig,
}

impl LintdiffConfig {
    pub fn effective(&self) -> EffectiveConfig {
        let profile = self.profile.clone().unwrap_or_default();

        let fail_on = self.fail_on.clone().unwrap_or_else(|| match profile {
            Profile::Default => FailOn::Error,
            Profile::Strict => FailOn::Warn,
            Profile::Advisory => FailOn::Never,
        });

        EffectiveConfig {
            profile,
            fail_on,
            max_findings: self.max_findings.unwrap_or(200),
            max_annotations: self.max_annotations.unwrap_or(50),
            workspace_only: self.workspace_only.unwrap_or(true),
            filter: self.filter.clone(),
            provenance: self.provenance.clone(),
        }
    }
}
