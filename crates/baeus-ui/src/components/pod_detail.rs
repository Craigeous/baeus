//! Typed structs for rich Pod detail views.
//!
//! All data is extracted from the raw `serde_json::Value` already cached in
//! `resource_detail_data` — no additional K8s API calls are required.

/// Top-level pod detail data extracted from a Pod JSON object.
#[derive(Debug, Clone)]
pub struct PodDetailData {
    pub containers: Vec<ContainerDetail>,
    pub init_containers: Vec<ContainerDetail>,
    pub volumes: Vec<VolumeDetail>,
    pub tolerations: Vec<TolerationDetail>,
    pub node_selector: Vec<(String, String)>,
    pub annotations: Vec<(String, String)>,
    /// Raw JSON string of the affinity block (if any).
    pub affinity_json: Option<String>,
    pub host_ip: String,
    pub dns_policy: String,
    pub priority_class: String,
    pub scheduler_name: String,
    pub termination_grace_period: String,
}

/// Full detail for a single container (regular or init).
#[derive(Debug, Clone)]
pub struct ContainerDetail {
    pub name: String,
    pub image: String,
    pub image_pull_policy: String,
    pub ports: Vec<PortDetail>,
    pub env_vars: Vec<EnvVarDetail>,
    pub volume_mounts: Vec<VolumeMountDetail>,
    pub resources: ContainerResources,
    pub liveness_probe: Option<ProbeDetail>,
    pub readiness_probe: Option<ProbeDetail>,
    pub startup_probe: Option<ProbeDetail>,
    pub command: Vec<String>,
    pub args: Vec<String>,
    pub working_dir: String,
    pub security_context: Option<SecurityContextDetail>,
    pub state: ContainerStateDetail,
    pub ready: bool,
    pub restart_count: i64,
}

/// A single container port mapping.
#[derive(Debug, Clone)]
pub struct PortDetail {
    pub name: String,
    pub container_port: i64,
    pub protocol: String,
    pub host_port: Option<i64>,
}

/// An environment variable — either a direct value or a reference description.
#[derive(Debug, Clone)]
pub struct EnvVarDetail {
    pub name: String,
    /// Direct value (empty string if sourced from a ref).
    pub value: String,
    /// Human-readable description of the source, e.g. "configMapKeyRef: my-cm.key"
    pub value_from: String,
}

/// A volume mount inside a container.
#[derive(Debug, Clone)]
pub struct VolumeMountDetail {
    pub name: String,
    pub mount_path: String,
    pub sub_path: String,
    pub read_only: bool,
}

/// Resource requests and limits for a container.
#[derive(Debug, Clone)]
pub struct ContainerResources {
    pub requests_cpu: String,
    pub requests_memory: String,
    pub limits_cpu: String,
    pub limits_memory: String,
}

/// A probe definition (liveness, readiness, or startup).
#[derive(Debug, Clone)]
pub struct ProbeDetail {
    pub probe_type: String,
    /// Human-readable detail, e.g. "HTTP GET :8080/healthz" or "exec: [cat, /tmp/healthy]"
    pub detail: String,
    pub initial_delay: i64,
    pub period: i64,
    pub timeout: i64,
    pub success_threshold: i64,
    pub failure_threshold: i64,
}

/// Security context for a container.
#[derive(Debug, Clone)]
pub struct SecurityContextDetail {
    pub run_as_user: Option<i64>,
    pub run_as_group: Option<i64>,
    pub run_as_non_root: Option<bool>,
    pub read_only_root_fs: Option<bool>,
    pub privileged: Option<bool>,
    pub caps_add: Vec<String>,
    pub caps_drop: Vec<String>,
}

/// Container state from the pod status.
#[derive(Debug, Clone)]
pub enum ContainerStateDetail {
    Running { started_at: String },
    Waiting { reason: String, message: String },
    Terminated { reason: String, exit_code: i64, finished_at: String },
    Unknown,
}

/// A pod volume definition.
#[derive(Debug, Clone)]
pub struct VolumeDetail {
    pub name: String,
    /// High-level type name, e.g. "configMap", "secret", "emptyDir", etc.
    pub volume_type: String,
    /// Human-readable detail, e.g. "my-config-map" or "medium: Memory"
    pub type_detail: String,
}

/// A pod toleration.
#[derive(Debug, Clone)]
pub struct TolerationDetail {
    pub key: String,
    pub operator: String,
    pub value: String,
    pub effect: String,
    pub toleration_seconds: Option<i64>,
}
