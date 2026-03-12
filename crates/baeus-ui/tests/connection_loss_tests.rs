// T363: Connection loss detection tests
//
// Tests for the `is_connection_error` helper function and connection state tracking.

use baeus_ui::layout::app_shell::is_connection_error;

// ===========================================================================
// 1. is_connection_error: Positive matches
// ===========================================================================

#[test]
fn test_connection_refused() {
    assert!(is_connection_error("Connection refused"));
    assert!(is_connection_error("tcp connection refused"));
    assert!(is_connection_error("error: Connection refused (os error 111)"));
}

#[test]
fn test_connection_reset() {
    assert!(is_connection_error("Connection reset by peer"));
    assert!(is_connection_error("connection reset"));
}

#[test]
fn test_timeout_errors() {
    assert!(is_connection_error("connection timed out"));
    assert!(is_connection_error("Operation timed out"));
    assert!(is_connection_error("request timeout after 30s"));
    assert!(is_connection_error("Timeout waiting for response"));
}

#[test]
fn test_network_unreachable() {
    assert!(is_connection_error("Network unreachable"));
    assert!(is_connection_error("network is unreachable"));
    assert!(is_connection_error("error: Network unreachable (os error 101)"));
}

#[test]
fn test_no_route_to_host() {
    assert!(is_connection_error("No route to host"));
    assert!(is_connection_error("no route to host (os error 113)"));
}

#[test]
fn test_dns_errors() {
    assert!(is_connection_error("DNS error: could not resolve"));
    assert!(is_connection_error("DNS resolution failed for api.cluster.local"));
    assert!(is_connection_error("Name or service not known"));
}

#[test]
fn test_broken_pipe() {
    assert!(is_connection_error("Broken pipe"));
    assert!(is_connection_error("broken pipe (os error 32)"));
}

#[test]
fn test_connection_closed() {
    assert!(is_connection_error("Connection closed before message completed"));
    assert!(is_connection_error("connection closed"));
}

#[test]
fn test_eof_error() {
    assert!(is_connection_error("unexpected EOF during handshake"));
    assert!(is_connection_error("eof"));
}

#[test]
fn test_hyper_error() {
    assert!(is_connection_error("hyper::Error(Connect, ...)"));
    assert!(is_connection_error("hyper::error: connection error"));
}

#[test]
fn test_connect_error() {
    assert!(is_connection_error("connect error: Connection refused"));
    assert!(is_connection_error("tcp connect error: timeout"));
}

// ===========================================================================
// 2. is_connection_error: Negative matches (non-connection errors)
// ===========================================================================

#[test]
fn test_not_found_is_not_connection_error() {
    assert!(!is_connection_error("404 Not Found"));
    assert!(!is_connection_error("resource pods/my-pod not found"));
}

#[test]
fn test_forbidden_is_not_connection_error() {
    assert!(!is_connection_error("403 Forbidden"));
    assert!(!is_connection_error("pods is forbidden: User cannot list resource"));
}

#[test]
fn test_unauthorized_is_not_connection_error() {
    assert!(!is_connection_error("401 Unauthorized"));
    assert!(!is_connection_error("Unauthorized: token expired"));
}

#[test]
fn test_conflict_is_not_connection_error() {
    assert!(!is_connection_error("409 Conflict: resource already exists"));
}

#[test]
fn test_validation_error_is_not_connection_error() {
    assert!(!is_connection_error("422 Unprocessable Entity: spec.replicas must be >= 0"));
}

#[test]
fn test_internal_server_error_is_not_connection_error() {
    assert!(!is_connection_error("500 Internal Server Error"));
}

#[test]
fn test_empty_string_is_not_connection_error() {
    assert!(!is_connection_error(""));
}

#[test]
fn test_generic_error_is_not_connection_error() {
    assert!(!is_connection_error("something went wrong"));
    assert!(!is_connection_error("unknown error"));
}

// ===========================================================================
// 3. is_connection_error: Case insensitivity
// ===========================================================================

#[test]
fn test_case_insensitivity() {
    assert!(is_connection_error("CONNECTION REFUSED"));
    assert!(is_connection_error("Timeout"));
    assert!(is_connection_error("NETWORK UNREACHABLE"));
    assert!(is_connection_error("DNS ERROR: lookup failed"));
}
