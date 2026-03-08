use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FailOn {
    #[default]
    Error,
    Warn,
    Never,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Profile {
    /// Favor signal and stability. Default.
    #[default]
    Default,
    /// Fail on warnings.
    Strict,
    /// Never fail. Useful for advisory runs.
    Advisory,
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct FeatureFlags {
    #[serde(default = "default_true")]
    pub prefer_primary_spans: bool,
    #[serde(default = "default_true")]
    pub path_filters: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            prefer_primary_spans: true,
            path_filters: true,
        }
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

    #[serde(default)]
    pub feature_flags: FeatureFlags,
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
    pub feature_flags: FeatureFlags,
}

impl LintdiffConfig {
    pub fn effective(&self) -> EffectiveConfig {
        let profile = self.profile.clone().unwrap_or_default();

        let fail_on = self.fail_on.clone().unwrap_or(match profile {
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
            feature_flags: self.feature_flags.clone(),
        }
    }
}
