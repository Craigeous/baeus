//! EKS Wizard state types and step definitions.
//!
//! The wizard guides users through AWS authentication and EKS cluster discovery.

use baeus_core::aws_eks::{
    AwsAuthMethod, AwsSession, EksAuthState, EksCluster, SsoAccount, SsoDeviceAuth, SsoRole,
};
use gpui::{Entity, Subscription};
use gpui_component::input::InputState;
use std::collections::{HashMap, HashSet};
use zeroize::Zeroize;

/// Steps of the EKS wizard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EksWizardStep {
    /// Choose authentication method (SSO, Access Key, Assume Role).
    ChooseAuthMethod,
    /// Enter SSO start URL and region.
    SsoConfig,
    /// Waiting for browser authorisation — shows user code and link.
    SsoDeviceAuth,
    /// Select an account from the SSO portal.
    SsoAccountSelection,
    /// Select a role within the chosen account.
    SsoRoleSelection,
    /// Enter an IAM role ARN to assume (e.g. avengers, defenders, watchers).
    /// This step appears after SSO auth when the SSO role itself doesn't have k8s access.
    AssumeIamRole,
    /// Enter access key ID, secret, optional session token, and region.
    AccessKeyConfig,
    /// Enter role ARN, optional external ID, and region.
    AssumeRoleConfig,
    /// Select which AWS regions to scan for EKS clusters.
    RegionSelection,
    /// Discovery in progress — shows progress bar.
    Discovering,
    /// Show discovered clusters and let the user pick which to connect.
    ClusterResults,
}

/// Full wizard state. Stored as `Option<EksWizardState>` on AppShell.
pub struct EksWizardState {
    pub step: EksWizardStep,
    pub auth_method: AwsAuthMethod,
    pub auth_state: EksAuthState,

    // SSO fields
    pub sso_start_url: String,
    pub sso_region: String,
    pub sso_client_id: Option<String>,
    pub sso_client_secret: Option<String>,
    pub sso_device_auth: Option<SsoDeviceAuth>,
    pub sso_access_token: Option<String>,
    pub sso_accounts: Vec<SsoAccount>,
    pub sso_selected_account: Option<SsoAccount>,
    pub sso_roles: Vec<SsoRole>,
    pub sso_selected_role: Option<SsoRole>,

    // Access key fields
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: String,
    pub access_key_region: String,

    // Assume role fields
    pub role_arn: String,
    pub external_id: String,
    pub assume_role_region: String,

    // IAM role to assume after SSO auth (for k8s access)
    pub iam_role_arn: String,

    // Session (set after successful auth — may be replaced by assumed role session)
    pub session: Option<AwsSession>,
    // Original SSO credentials (preserved for per-cluster role assumption when
    // the step-2 assumed role can't chain to another role)
    pub original_sso_credentials: Option<aws_credential_types::Credentials>,

    // Region selection
    pub selected_regions: HashSet<String>,

    // Discovery
    pub discovered_clusters: Vec<EksCluster>,
    pub discovery_progress: (usize, usize), // (completed, total)
    pub selected_cluster_indices: HashSet<usize>,
    /// Per-cluster IAM role ARN overrides. Key = cluster index.
    /// If set, this role is assumed (using the SSO session) for this specific cluster.
    /// Falls back to `iam_role_arn` if not set for a cluster.
    pub per_cluster_roles: HashMap<usize, String>,

    // Error messages
    pub error: Option<String>,

    // Search/filter text for list selection steps.
    pub filter_text: String,

    // GPUI input entities for text fields, keyed by field name.
    pub inputs: HashMap<String, Entity<InputState>>,
    // Subscriptions for input change events (kept alive to receive events).
    pub _input_subs: Vec<Subscription>,
}

impl Default for EksWizardState {
    fn default() -> Self {
        let mut selected_regions = HashSet::new();
        for region in &["us-east-1", "us-east-2", "us-west-1", "us-west-2"] {
            selected_regions.insert((*region).to_string());
        }

        Self {
            step: EksWizardStep::ChooseAuthMethod,
            auth_method: AwsAuthMethod::Sso,
            auth_state: EksAuthState::Idle,

            sso_start_url: String::new(),
            sso_region: "us-east-1".to_string(),
            sso_client_id: None,
            sso_client_secret: None,
            sso_device_auth: None,
            sso_access_token: None,
            sso_accounts: Vec::new(),
            sso_selected_account: None,
            sso_roles: Vec::new(),
            sso_selected_role: None,

            access_key_id: String::new(),
            secret_access_key: String::new(),
            session_token: String::new(),
            access_key_region: "us-east-1".to_string(),

            role_arn: String::new(),
            external_id: String::new(),
            assume_role_region: "us-east-1".to_string(),

            iam_role_arn: String::new(),

            session: None,
            original_sso_credentials: None,

            selected_regions,

            discovered_clusters: Vec::new(),
            discovery_progress: (0, 0),
            selected_cluster_indices: HashSet::new(),
            per_cluster_roles: HashMap::new(),

            error: None,

            filter_text: String::new(),

            inputs: HashMap::new(),
            _input_subs: Vec::new(),
        }
    }
}

impl EksWizardState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the "Next" / "Continue" button should be enabled for the current step.
    pub fn can_advance(&self) -> bool {
        match self.step {
            EksWizardStep::ChooseAuthMethod => true,
            EksWizardStep::SsoConfig => {
                !self.sso_start_url.trim().is_empty() && !self.sso_region.trim().is_empty()
            }
            EksWizardStep::SsoDeviceAuth => false,
            EksWizardStep::SsoAccountSelection => self.sso_selected_account.is_some(),
            EksWizardStep::SsoRoleSelection => self.sso_selected_role.is_some(),
            EksWizardStep::AssumeIamRole => true, // can skip if SSO role has direct access
            EksWizardStep::AccessKeyConfig => {
                !self.access_key_id.trim().is_empty()
                    && !self.secret_access_key.trim().is_empty()
                    && !self.access_key_region.trim().is_empty()
            }
            EksWizardStep::AssumeRoleConfig => {
                !self.role_arn.trim().is_empty() && !self.assume_role_region.trim().is_empty()
            }
            EksWizardStep::RegionSelection => !self.selected_regions.is_empty(),
            EksWizardStep::Discovering => false,
            EksWizardStep::ClusterResults => !self.selected_cluster_indices.is_empty(),
        }
    }
}

impl Drop for EksWizardState {
    fn drop(&mut self) {
        self.secret_access_key.zeroize();
        self.session_token.zeroize();
        self.iam_role_arn.zeroize();
        if let Some(ref mut secret) = self.sso_client_secret {
            secret.zeroize();
        }
        if let Some(ref mut token) = self.sso_access_token {
            token.zeroize();
        }
    }
}
