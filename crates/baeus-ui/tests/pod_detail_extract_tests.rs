// Pod detail extraction tests
//
// Tests for extract_pod_detail() and all sub-extraction functions in json_extract.rs.
// Uses a rich pod JSON fixture with multiple containers, init containers, volumes, etc.

use baeus_ui::components::json_extract;
use baeus_ui::components::pod_detail::*;

// ---------------------------------------------------------------------------
// Fixture: rich pod JSON
// ---------------------------------------------------------------------------

fn rich_pod_json() -> serde_json::Value {
    serde_json::json!({
        "apiVersion": "v1",
        "kind": "Pod",
        "metadata": {
            "name": "web-server-abc123",
            "namespace": "production",
            "uid": "12345-abcde",
            "creationTimestamp": "2025-01-15T10:00:00Z",
            "resourceVersion": "987654",
            "labels": {
                "app": "web-server",
                "version": "v2",
                "tier": "frontend"
            },
            "annotations": {
                "prometheus.io/scrape": "true",
                "prometheus.io/port": "9090"
            },
            "ownerReferences": [{
                "kind": "ReplicaSet",
                "name": "web-server-abc"
            }]
        },
        "spec": {
            "nodeName": "node-1",
            "serviceAccountName": "web-sa",
            "restartPolicy": "Always",
            "dnsPolicy": "ClusterFirst",
            "priorityClassName": "high-priority",
            "schedulerName": "default-scheduler",
            "terminationGracePeriodSeconds": 30,
            "nodeSelector": {
                "disktype": "ssd",
                "region": "us-west-2"
            },
            "tolerations": [
                {
                    "key": "node.kubernetes.io/not-ready",
                    "operator": "Exists",
                    "effect": "NoExecute",
                    "tolerationSeconds": 300
                },
                {
                    "key": "dedicated",
                    "operator": "Equal",
                    "value": "web",
                    "effect": "NoSchedule"
                }
            ],
            "affinity": {
                "nodeAffinity": {
                    "requiredDuringSchedulingIgnoredDuringExecution": {
                        "nodeSelectorTerms": [{
                            "matchExpressions": [{
                                "key": "kubernetes.io/os",
                                "operator": "In",
                                "values": ["linux"]
                            }]
                        }]
                    }
                }
            },
            "containers": [
                {
                    "name": "nginx",
                    "image": "nginx:1.25",
                    "imagePullPolicy": "IfNotPresent",
                    "ports": [
                        {"name": "http", "containerPort": 80, "protocol": "TCP"},
                        {"name": "metrics", "containerPort": 9090, "protocol": "TCP", "hostPort": 9090}
                    ],
                    "env": [
                        {"name": "ENV", "value": "production"},
                        {"name": "DB_HOST", "valueFrom": {"configMapKeyRef": {"name": "db-config", "key": "host"}}},
                        {"name": "DB_PASS", "valueFrom": {"secretKeyRef": {"name": "db-secret", "key": "password"}}},
                        {"name": "POD_NAME", "valueFrom": {"fieldRef": {"fieldPath": "metadata.name"}}}
                    ],
                    "volumeMounts": [
                        {"name": "config", "mountPath": "/etc/nginx/conf.d", "readOnly": true},
                        {"name": "data", "mountPath": "/var/www", "subPath": "html"}
                    ],
                    "resources": {
                        "requests": {"cpu": "100m", "memory": "128Mi"},
                        "limits": {"cpu": "500m", "memory": "512Mi"}
                    },
                    "livenessProbe": {
                        "httpGet": {"path": "/healthz", "port": 8080},
                        "initialDelaySeconds": 10,
                        "periodSeconds": 15,
                        "timeoutSeconds": 3,
                        "successThreshold": 1,
                        "failureThreshold": 5
                    },
                    "readinessProbe": {
                        "tcpSocket": {"port": 80},
                        "periodSeconds": 5
                    },
                    "command": ["/bin/sh"],
                    "args": ["-c", "nginx -g 'daemon off;'"],
                    "workingDir": "/app",
                    "securityContext": {
                        "runAsUser": 1000,
                        "runAsGroup": 1000,
                        "runAsNonRoot": true,
                        "readOnlyRootFilesystem": true,
                        "privileged": false,
                        "capabilities": {
                            "add": ["NET_BIND_SERVICE"],
                            "drop": ["ALL"]
                        }
                    }
                },
                {
                    "name": "sidecar",
                    "image": "fluentd:v1.16",
                    "imagePullPolicy": "Always",
                    "resources": {
                        "requests": {"cpu": "50m", "memory": "64Mi"}
                    }
                }
            ],
            "initContainers": [
                {
                    "name": "init-db",
                    "image": "busybox:1.36",
                    "command": ["sh", "-c", "until nslookup db; do sleep 2; done"]
                }
            ],
            "volumes": [
                {"name": "config", "configMap": {"name": "nginx-config"}},
                {"name": "data", "persistentVolumeClaim": {"claimName": "web-data-pvc"}},
                {"name": "tmp", "emptyDir": {"medium": "Memory"}},
                {"name": "secrets", "secret": {"secretName": "tls-certs"}}
            ]
        },
        "status": {
            "phase": "Running",
            "podIP": "10.0.0.42",
            "hostIP": "192.168.1.100",
            "qosClass": "Burstable",
            "containerStatuses": [
                {
                    "name": "nginx",
                    "ready": true,
                    "restartCount": 2,
                    "state": {"running": {"startedAt": "2025-01-15T10:01:00Z"}}
                },
                {
                    "name": "sidecar",
                    "ready": false,
                    "restartCount": 0,
                    "state": {"waiting": {"reason": "CrashLoopBackOff", "message": "back-off 5m0s restarting"}}
                }
            ],
            "initContainerStatuses": [
                {
                    "name": "init-db",
                    "ready": true,
                    "restartCount": 0,
                    "state": {"terminated": {"reason": "Completed", "exitCode": 0, "finishedAt": "2025-01-15T10:00:50Z"}}
                }
            ],
            "conditions": [
                {"type": "Ready", "status": "True", "reason": "PodReady", "message": "", "lastTransitionTime": "2025-01-15T10:01:05Z"},
                {"type": "Initialized", "status": "True", "reason": "", "message": "", "lastTransitionTime": "2025-01-15T10:00:55Z"}
            ]
        }
    })
}

// ---------------------------------------------------------------------------
// Top-level extraction
// ---------------------------------------------------------------------------

#[test]
fn test_extract_pod_detail_basic_fields() {
    let json = rich_pod_json();
    let pod = json_extract::extract_pod_detail(&json);

    assert_eq!(pod.host_ip, "192.168.1.100");
    assert_eq!(pod.dns_policy, "ClusterFirst");
    assert_eq!(pod.priority_class, "high-priority");
    assert_eq!(pod.scheduler_name, "default-scheduler");
    assert_eq!(pod.termination_grace_period, "30s");
}

#[test]
fn test_extract_pod_detail_container_count() {
    let json = rich_pod_json();
    let pod = json_extract::extract_pod_detail(&json);

    assert_eq!(pod.containers.len(), 2);
    assert_eq!(pod.init_containers.len(), 1);
}

#[test]
fn test_extract_pod_detail_annotations() {
    let json = rich_pod_json();
    let pod = json_extract::extract_pod_detail(&json);

    assert_eq!(pod.annotations.len(), 2);
    assert!(pod.annotations.iter().any(|(k, v)| k == "prometheus.io/scrape" && v == "true"));
}

#[test]
fn test_extract_pod_detail_node_selector() {
    let json = rich_pod_json();
    let pod = json_extract::extract_pod_detail(&json);

    assert_eq!(pod.node_selector.len(), 2);
    assert!(pod.node_selector.iter().any(|(k, v)| k == "disktype" && v == "ssd"));
}

#[test]
fn test_extract_pod_detail_affinity_present() {
    let json = rich_pod_json();
    let pod = json_extract::extract_pod_detail(&json);

    assert!(pod.affinity_json.is_some());
    let affinity = pod.affinity_json.unwrap();
    assert!(affinity.contains("nodeAffinity"));
}

// ---------------------------------------------------------------------------
// Container detail extraction
// ---------------------------------------------------------------------------

#[test]
fn test_container_basic_fields() {
    let json = rich_pod_json();
    let pod = json_extract::extract_pod_detail(&json);
    let nginx = &pod.containers[0];

    assert_eq!(nginx.name, "nginx");
    assert_eq!(nginx.image, "nginx:1.25");
    assert_eq!(nginx.image_pull_policy, "IfNotPresent");
}

#[test]
fn test_container_ports() {
    let json = rich_pod_json();
    let pod = json_extract::extract_pod_detail(&json);
    let nginx = &pod.containers[0];

    assert_eq!(nginx.ports.len(), 2);
    assert_eq!(nginx.ports[0].name, "http");
    assert_eq!(nginx.ports[0].container_port, 80);
    assert_eq!(nginx.ports[0].protocol, "TCP");
    assert!(nginx.ports[0].host_port.is_none());
    assert_eq!(nginx.ports[1].host_port, Some(9090));
}

#[test]
fn test_container_env_vars() {
    let json = rich_pod_json();
    let pod = json_extract::extract_pod_detail(&json);
    let nginx = &pod.containers[0];

    assert_eq!(nginx.env_vars.len(), 4);

    // Direct value
    let env_direct = &nginx.env_vars[0];
    assert_eq!(env_direct.name, "ENV");
    assert_eq!(env_direct.value, "production");
    assert!(env_direct.value_from.is_empty());

    // ConfigMap ref
    let env_cm = &nginx.env_vars[1];
    assert_eq!(env_cm.name, "DB_HOST");
    assert!(env_cm.value_from.contains("configMapKeyRef"));
    assert!(env_cm.value_from.contains("db-config.host"));

    // Secret ref
    let env_sec = &nginx.env_vars[2];
    assert_eq!(env_sec.name, "DB_PASS");
    assert!(env_sec.value_from.contains("secretKeyRef"));

    // Field ref
    let env_field = &nginx.env_vars[3];
    assert_eq!(env_field.name, "POD_NAME");
    assert!(env_field.value_from.contains("fieldRef"));
}

#[test]
fn test_container_volume_mounts() {
    let json = rich_pod_json();
    let pod = json_extract::extract_pod_detail(&json);
    let nginx = &pod.containers[0];

    assert_eq!(nginx.volume_mounts.len(), 2);
    assert_eq!(nginx.volume_mounts[0].name, "config");
    assert_eq!(nginx.volume_mounts[0].mount_path, "/etc/nginx/conf.d");
    assert!(nginx.volume_mounts[0].read_only);

    assert_eq!(nginx.volume_mounts[1].sub_path, "html");
    assert!(!nginx.volume_mounts[1].read_only);
}

#[test]
fn test_container_resources() {
    let json = rich_pod_json();
    let pod = json_extract::extract_pod_detail(&json);
    let nginx = &pod.containers[0];

    assert_eq!(nginx.resources.requests_cpu, "100m");
    assert_eq!(nginx.resources.requests_memory, "128Mi");
    assert_eq!(nginx.resources.limits_cpu, "500m");
    assert_eq!(nginx.resources.limits_memory, "512Mi");
}

#[test]
fn test_container_resources_partial() {
    let json = rich_pod_json();
    let pod = json_extract::extract_pod_detail(&json);
    let sidecar = &pod.containers[1];

    assert_eq!(sidecar.resources.requests_cpu, "50m");
    assert_eq!(sidecar.resources.requests_memory, "64Mi");
    assert_eq!(sidecar.resources.limits_cpu, "\u{2014}");
    assert_eq!(sidecar.resources.limits_memory, "\u{2014}");
}

#[test]
fn test_container_probes() {
    let json = rich_pod_json();
    let pod = json_extract::extract_pod_detail(&json);
    let nginx = &pod.containers[0];

    // Liveness: HTTP GET
    let liveness = nginx.liveness_probe.as_ref().unwrap();
    assert_eq!(liveness.probe_type, "liveness");
    assert!(liveness.detail.contains("HTTP GET"));
    assert!(liveness.detail.contains("8080"));
    assert!(liveness.detail.contains("/healthz"));
    assert_eq!(liveness.initial_delay, 10);
    assert_eq!(liveness.period, 15);
    assert_eq!(liveness.timeout, 3);
    assert_eq!(liveness.failure_threshold, 5);

    // Readiness: TCP
    let readiness = nginx.readiness_probe.as_ref().unwrap();
    assert_eq!(readiness.probe_type, "readiness");
    assert!(readiness.detail.contains("TCP"));
    assert!(readiness.detail.contains("80"));

    // No startup probe
    assert!(nginx.startup_probe.is_none());
}

#[test]
fn test_container_command_and_args() {
    let json = rich_pod_json();
    let pod = json_extract::extract_pod_detail(&json);
    let nginx = &pod.containers[0];

    assert_eq!(nginx.command, vec!["/bin/sh"]);
    assert_eq!(nginx.args, vec!["-c", "nginx -g 'daemon off;'"]);
    assert_eq!(nginx.working_dir, "/app");
}

#[test]
fn test_container_security_context() {
    let json = rich_pod_json();
    let pod = json_extract::extract_pod_detail(&json);
    let nginx = &pod.containers[0];

    let sc = nginx.security_context.as_ref().unwrap();
    assert_eq!(sc.run_as_user, Some(1000));
    assert_eq!(sc.run_as_group, Some(1000));
    assert_eq!(sc.run_as_non_root, Some(true));
    assert_eq!(sc.read_only_root_fs, Some(true));
    assert_eq!(sc.privileged, Some(false));
    assert_eq!(sc.caps_add, vec!["NET_BIND_SERVICE"]);
    assert_eq!(sc.caps_drop, vec!["ALL"]);
}

#[test]
fn test_container_state_running() {
    let json = rich_pod_json();
    let pod = json_extract::extract_pod_detail(&json);
    let nginx = &pod.containers[0];

    assert!(nginx.ready);
    assert_eq!(nginx.restart_count, 2);
    match &nginx.state {
        ContainerStateDetail::Running { started_at } => {
            assert!(started_at.contains("2025-01-15"));
        }
        _ => panic!("Expected Running state"),
    }
}

#[test]
fn test_container_state_waiting() {
    let json = rich_pod_json();
    let pod = json_extract::extract_pod_detail(&json);
    let sidecar = &pod.containers[1];

    assert!(!sidecar.ready);
    assert_eq!(sidecar.restart_count, 0);
    match &sidecar.state {
        ContainerStateDetail::Waiting { reason, message } => {
            assert_eq!(reason, "CrashLoopBackOff");
            assert!(message.contains("back-off"));
        }
        _ => panic!("Expected Waiting state"),
    }
}

#[test]
fn test_init_container_state_terminated() {
    let json = rich_pod_json();
    let pod = json_extract::extract_pod_detail(&json);
    let init = &pod.init_containers[0];

    assert_eq!(init.name, "init-db");
    assert_eq!(init.image, "busybox:1.36");
    match &init.state {
        ContainerStateDetail::Terminated { reason, exit_code, .. } => {
            assert_eq!(reason, "Completed");
            assert_eq!(*exit_code, 0);
        }
        _ => panic!("Expected Terminated state"),
    }
}

// ---------------------------------------------------------------------------
// Volumes extraction
// ---------------------------------------------------------------------------

#[test]
fn test_volumes_extraction() {
    let json = rich_pod_json();
    let pod = json_extract::extract_pod_detail(&json);

    assert_eq!(pod.volumes.len(), 4);

    let config_vol = pod.volumes.iter().find(|v| v.name == "config").unwrap();
    assert_eq!(config_vol.volume_type, "configMap");
    assert_eq!(config_vol.type_detail, "nginx-config");

    let data_vol = pod.volumes.iter().find(|v| v.name == "data").unwrap();
    assert_eq!(data_vol.volume_type, "persistentVolumeClaim");
    assert_eq!(data_vol.type_detail, "web-data-pvc");

    let tmp_vol = pod.volumes.iter().find(|v| v.name == "tmp").unwrap();
    assert_eq!(tmp_vol.volume_type, "emptyDir");
    assert!(tmp_vol.type_detail.contains("Memory"));

    let sec_vol = pod.volumes.iter().find(|v| v.name == "secrets").unwrap();
    assert_eq!(sec_vol.volume_type, "secret");
    assert_eq!(sec_vol.type_detail, "tls-certs");
}

// ---------------------------------------------------------------------------
// Tolerations extraction
// ---------------------------------------------------------------------------

#[test]
fn test_tolerations_extraction() {
    let json = rich_pod_json();
    let pod = json_extract::extract_pod_detail(&json);

    assert_eq!(pod.tolerations.len(), 2);

    let tol1 = &pod.tolerations[0];
    assert_eq!(tol1.key, "node.kubernetes.io/not-ready");
    assert_eq!(tol1.operator, "Exists");
    assert_eq!(tol1.effect, "NoExecute");
    assert_eq!(tol1.toleration_seconds, Some(300));

    let tol2 = &pod.tolerations[1];
    assert_eq!(tol2.key, "dedicated");
    assert_eq!(tol2.operator, "Equal");
    assert_eq!(tol2.value, "web");
    assert_eq!(tol2.effect, "NoSchedule");
    assert!(tol2.toleration_seconds.is_none());
}

// ---------------------------------------------------------------------------
// Enhanced detail properties
// ---------------------------------------------------------------------------

#[test]
fn test_extract_detail_properties_pod_enhanced() {
    let json = rich_pod_json();
    let props = json_extract::extract_detail_properties("Pod", &json);

    let find = |key: &str| props.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str());

    assert_eq!(find("Host IP"), Some("192.168.1.100"));
    assert_eq!(find("DNS Policy"), Some("ClusterFirst"));
    assert_eq!(find("Priority Class"), Some("high-priority"));
    assert_eq!(find("Scheduler"), Some("default-scheduler"));
    assert_eq!(find("Termination Grace Period"), Some("30s"));
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_extract_pod_detail_minimal_json() {
    let json = serde_json::json!({
        "metadata": {"name": "minimal-pod"},
        "spec": {},
        "status": {}
    });
    let pod = json_extract::extract_pod_detail(&json);

    assert!(pod.containers.is_empty());
    assert!(pod.init_containers.is_empty());
    assert!(pod.volumes.is_empty());
    assert!(pod.tolerations.is_empty());
    assert!(pod.node_selector.is_empty());
    assert!(pod.annotations.is_empty());
    assert!(pod.affinity_json.is_none());
    assert_eq!(pod.host_ip, "\u{2014}");
    assert_eq!(pod.dns_policy, "\u{2014}");
}

#[test]
fn test_extract_pod_detail_no_status() {
    let json = serde_json::json!({
        "metadata": {"name": "no-status-pod"},
        "spec": {
            "containers": [{
                "name": "app",
                "image": "myapp:latest"
            }]
        }
    });
    let pod = json_extract::extract_pod_detail(&json);

    assert_eq!(pod.containers.len(), 1);
    let c = &pod.containers[0];
    assert_eq!(c.name, "app");
    assert!(!c.ready);
    assert_eq!(c.restart_count, 0);
    match &c.state {
        ContainerStateDetail::Unknown => {}
        _ => panic!("Expected Unknown state when no status"),
    }
}

#[test]
fn test_container_no_security_context() {
    let json = rich_pod_json();
    let pod = json_extract::extract_pod_detail(&json);
    let sidecar = &pod.containers[1];

    assert!(sidecar.security_context.is_none());
}

#[test]
fn test_container_no_probes() {
    let json = rich_pod_json();
    let pod = json_extract::extract_pod_detail(&json);
    let sidecar = &pod.containers[1];

    assert!(sidecar.liveness_probe.is_none());
    assert!(sidecar.readiness_probe.is_none());
    assert!(sidecar.startup_probe.is_none());
}

#[test]
fn test_volume_types_coverage() {
    let json = serde_json::json!({
        "metadata": {"name": "vol-test-pod"},
        "spec": {
            "volumes": [
                {"name": "hp", "hostPath": {"path": "/var/log"}},
                {"name": "proj", "projected": {"sources": []}},
                {"name": "dapi", "downwardAPI": {"items": []}},
                {"name": "nfs-vol", "nfs": {"server": "nfs.example.com", "path": "/export"}},
                {"name": "csi-vol", "csi": {"driver": "ebs.csi.aws.com"}},
                {"name": "unknown-vol", "awsElasticBlockStore": {}}
            ]
        },
        "status": {}
    });
    let pod = json_extract::extract_pod_detail(&json);
    assert_eq!(pod.volumes.len(), 6);

    assert_eq!(pod.volumes[0].volume_type, "hostPath");
    assert_eq!(pod.volumes[0].type_detail, "/var/log");

    assert_eq!(pod.volumes[1].volume_type, "projected");

    assert_eq!(pod.volumes[2].volume_type, "downwardAPI");

    assert_eq!(pod.volumes[3].volume_type, "nfs");
    assert!(pod.volumes[3].type_detail.contains("nfs.example.com"));

    assert_eq!(pod.volumes[4].volume_type, "csi");
    assert!(pod.volumes[4].type_detail.contains("ebs.csi.aws.com"));

    assert_eq!(pod.volumes[5].volume_type, "unknown");
}

#[test]
fn test_probe_exec_type() {
    let json = serde_json::json!({
        "metadata": {"name": "exec-probe-pod"},
        "spec": {
            "containers": [{
                "name": "app",
                "image": "app:latest",
                "livenessProbe": {
                    "exec": {"command": ["cat", "/tmp/healthy"]},
                    "periodSeconds": 5
                }
            }]
        },
        "status": {}
    });
    let pod = json_extract::extract_pod_detail(&json);
    let probe = pod.containers[0].liveness_probe.as_ref().unwrap();
    assert!(probe.detail.contains("exec"));
    assert!(probe.detail.contains("cat"));
}

#[test]
fn test_probe_grpc_type() {
    let json = serde_json::json!({
        "metadata": {"name": "grpc-probe-pod"},
        "spec": {
            "containers": [{
                "name": "app",
                "image": "app:latest",
                "startupProbe": {
                    "grpc": {"port": 50051, "service": "health.v1"},
                    "initialDelaySeconds": 5
                }
            }]
        },
        "status": {}
    });
    let pod = json_extract::extract_pod_detail(&json);
    let probe = pod.containers[0].startup_probe.as_ref().unwrap();
    assert!(probe.detail.contains("gRPC"));
    assert!(probe.detail.contains("50051"));
    assert!(probe.detail.contains("health.v1"));
    assert_eq!(probe.initial_delay, 5);
}
