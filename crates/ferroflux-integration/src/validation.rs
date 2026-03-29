use std::collections::HashSet;
use std::path::PathBuf;
use ferroflux_security::signing::{verify_content, is_trusted_key};
use serde_json::json;

use crate::definition::{NodeDefinition, PlatformDefinition};

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct ValidationDiagnostic {
    pub severity: Severity,
    /// Short machine-readable rule name, e.g. "missing-exec-port"
    pub rule: &'static str,
    pub message: String,
    pub file: Option<PathBuf>,
}

impl ValidationDiagnostic {
    fn error(rule: &'static str, message: impl Into<String>) -> Self {
        Self { severity: Severity::Error, rule, message: message.into(), file: None }
    }

    fn warning(rule: &'static str, message: impl Into<String>) -> Self {
        Self { severity: Severity::Warning, rule, message: message.into(), file: None }
    }

    pub fn with_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.file = Some(path.into());
        self
    }
}

#[derive(Debug, Default)]
pub struct ValidationResult {
    pub diagnostics: Vec<ValidationDiagnostic>,
}

impl ValidationResult {
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == Severity::Error)
    }

    pub fn is_ok(&self) -> bool {
        !self.has_errors()
    }

    pub fn merge(&mut self, other: ValidationResult) {
        self.diagnostics.extend(other.diagnostics);
    }
}

/// Validate a single platform definition.
pub fn validate_platform(def: &PlatformDefinition) -> ValidationResult {
    let mut result = ValidationResult::default();

    // Rule 1: meta.type must be "Platform" (case-insensitive)
    if !def.meta.node_type.eq_ignore_ascii_case("Platform") {
        result.diagnostics.push(ValidationDiagnostic::error(
            "wrong-type",
            format!("meta.type must be 'Platform', got '{}'", def.meta.node_type),
        ));
    }

    // Rule 2: config.base_url should exist and have no trailing slash.
    // Warning only — some internal/meta platforms (e.g. "core") legitimately omit it.
    match def.config.get("base_url").and_then(|v| v.as_str()) {
        None => {
            result.diagnostics.push(ValidationDiagnostic::warning(
                "missing-base-url",
                "config.base_url is not set; HTTP integration platforms should define a base URL",
            ));
        }
        Some(url) if url.ends_with('/') => {
            result.diagnostics.push(ValidationDiagnostic::warning(
                "trailing-slash-base-url",
                format!("config.base_url should not have a trailing slash: '{url}'"),
            ));
        }
        _ => {}
    }

    // Rule 3: config.headers should exist for HTTP platforms.
    // Warning only — internal platforms may not need headers.
    if !def.config.contains_key("headers") {
        result.diagnostics.push(ValidationDiagnostic::warning(
            "missing-headers",
            "config.headers is not set; HTTP integration platforms should define headers (even an empty map)",
        ));
    }

    result
}

/// Validate a single node definition.
pub fn validate_node(def: &NodeDefinition) -> ValidationResult {
    let mut result = ValidationResult::default();

    // Rule 1: meta.type must be a known node type (case-insensitive)
    let valid_types = ["Action", "Trigger", "Utility"];
    let normalized_type = {
        let mut s = def.meta.node_type.clone();
        if let Some(c) = s.get_mut(0..1) {
            c.make_ascii_uppercase();
        }
        s
    };
    if !valid_types.contains(&normalized_type.as_str()) {
        result.diagnostics.push(ValidationDiagnostic::error(
            "unknown-node-type",
            format!(
                "meta.type '{}' is not valid; expected one of: {}",
                def.meta.node_type,
                valid_types.join(", ")
            ),
        ));
    }

    // Rule 2: Action nodes must declare their platform
    if normalized_type == "Action" && def.meta.platform.is_none() {
        result.diagnostics.push(ValidationDiagnostic::error(
            "missing-platform-ref",
            "Action nodes must set meta.platform",
        ));
    }

    // Node subtypes that legitimately deviate from the standard Exec→Success/Error pattern.
    // Router   — branches to named outputs (If/Else, Switch); no single Success/Error
    // Iterator — emits items one-by-one then Done; no standard completion port
    // Accumulator — stream-fed (no Exec input); fires when batch is full
    // Terminus — ends a sub-flow; no outputs at all
    let subtype = def.meta.node_subtype.as_deref().unwrap_or("").to_string();
    let is_special = matches!(
        subtype.as_str(),
        "Router" | "Iterator" | "Accumulator" | "Terminus"
    );

    // Rule 3: First input must be Exec (flow).
    // Exceptions:
    //   - Trigger nodes have no inputs (they start flows themselves)
    //   - Accumulator nodes take a stream item as their first input, not an Exec token
    let skip_exec_check = normalized_type == "Trigger" || subtype == "Accumulator";
    if !skip_exec_check {
        let first_input = def.interface.inputs.first();
        match first_input {
            None => {
                result.diagnostics.push(ValidationDiagnostic::error(
                    "missing-exec-port",
                    "interface.inputs must start with an Exec port of type 'flow'",
                ));
            }
            Some(port) if port.name != "Exec" || port.data_type != "flow" => {
                result.diagnostics.push(ValidationDiagnostic::error(
                    "missing-exec-port",
                    format!(
                        "first input must be name='Exec', type='flow'; got name='{}', type='{}'",
                        port.name, port.data_type
                    ),
                ));
            }
            _ => {}
        }
    }

    // Rule 4: Outputs must include a completed-flow port and an Error port.
    // "Success" is the canonical name; "Exec" is also accepted (used by older core nodes).
    // Special subtypes define their own output contracts — skip these checks for them.
    // Trigger nodes don't require Error — they start flows, not continue them.
    if !is_special {
        let has_success = def
            .interface
            .outputs
            .iter()
            .any(|p| (p.name == "Success" || p.name == "Exec") && p.data_type == "flow");
        let has_error = def
            .interface
            .outputs
            .iter()
            .any(|p| p.name == "Error" && p.data_type == "flow");

        if !has_success {
            result.diagnostics.push(ValidationDiagnostic::error(
                "missing-success-port",
                "interface.outputs must include a 'Success' (or 'Exec') port of type 'flow'",
            ));
        }
        if !has_error && normalized_type != "Trigger" {
            result.diagnostics.push(ValidationDiagnostic::error(
                "missing-error-port",
                "interface.outputs must include an 'Error' port of type 'flow'",
            ));
        }
    }

    // Rule 5: Every http_client step must have a non-empty returns block
    for step in &def.execution {
        if step.tool == "http_client" && step.returns.is_empty() {
            result.diagnostics.push(ValidationDiagnostic::error(
                "missing-returns",
                format!(
                    "http_client step '{}' has an empty returns block; add at least 'status' and 'body' mappings",
                    step.id
                ),
            ));
        }
    }

    // Rule 6 (warning): http_client URL should use {{ platform.base_url }}, not a hardcoded domain
    for step in &def.execution {
        if step.tool == "http_client"
            && let Some(url) = step.params.get("url").and_then(|v| v.as_str())
            && (url.starts_with("http://") || url.starts_with("https://"))
            && !url.contains("platform.base_url")
        {
            result.diagnostics.push(ValidationDiagnostic::warning(
                "hardcoded-url",
                format!(
                    "step '{}' uses a hardcoded URL '{url}'; use '{{{{ platform.base_url }}}}' instead",
                    step.id
                ),
            ));
        }
    }

    // Rule 7 (warning): meta.id should follow platform.category.verb namespacing
    if let Some(platform) = &def.meta.platform
        && normalized_type == "Action"
        && !def.meta.id.starts_with(&format!("{platform}."))
    {
        result.diagnostics.push(ValidationDiagnostic::warning(
            "id-not-namespaced",
            format!(
                "meta.id '{}' should start with '{platform}.' to follow the namespacing convention",
                def.meta.id
            ),
        ));
    }

    // Rule 8 (warning): execution steps should reference known tools
    const KNOWN_TOOLS: &[&str] = &[
        "http_client", "paginate", "json_query", "emit", "logic",
        "log", "sleep", "set_var", "get_var", "math", "rhai",
        "trace", "stats", "verify_signature", "transform",
        "switch", "agent", "aggregate", "split", "ferroflux:stats",
        "sql_query", "mongo_query", "redis_query",
    ];
    for step in &def.execution {
        if !KNOWN_TOOLS.contains(&step.tool.as_str()) {
            result.diagnostics.push(ValidationDiagnostic::warning(
                "unknown-tool",
                format!(
                    "execution step '{}' references unknown tool '{}'; known tools: {}",
                    step.id, step.tool, KNOWN_TOOLS.join(", ")
                ),
            ));
        }
    }

    // Rule 9: template syntax — every {{ must have a matching }}
    for step in &def.execution {
        check_template_syntax(&step.params, &step.id, &mut result);
    }

    // Rule 10: Signature Verification (Integrity & Trust)
    if let Some(sig) = &def.meta.signature {
        // Strip the signature from the content before verifying
        let mut content = json!(def);
        if let Some(obj) = content.as_object_mut() {
            if let Some(meta) = obj.get_mut("meta").and_then(|m| m.as_object_mut()) {
                meta.remove("signature");
            }
        }
        
        match verify_content(&content, sig) {
            Ok(_) => {
                if !is_trusted_key(&sig.public_key) {
                    result.diagnostics.push(ValidationDiagnostic::warning(
                        "untrusted-signer",
                        format!("Node is signed by an untrusted developer ({})", sig.signer_name),
                    ));
                }
            }
            Err(e) => {
                result.diagnostics.push(ValidationDiagnostic::error(
                    "invalid-signature",
                    format!("Cryptographic signature is invalid: {}", e),
                ));
            }
        }
    } else {
        result.diagnostics.push(ValidationDiagnostic::warning(
            "unsigned-node",
            "Node is not signed; its authenticity and integrity cannot be guaranteed",
        ));
    }

    // Rule 11: Permission Auditing
    for step in &def.execution {
        if step.tool == "http_client" {
            if let Some(url) = step.params.get("url").and_then(|v| v.as_str()) {
                // If it's a hardcoded external URL (not using platform.base_url)
                if (url.starts_with("http://") || url.starts_with("https://")) 
                    && !url.contains("platform.base_url") 
                {
                    let domain = url.split('/').nth(2).unwrap_or("");
                    let required_perm = format!("network:{}", domain);
                    
                    if !def.meta.permissions.contains(&required_perm) {
                        result.diagnostics.push(ValidationDiagnostic::error(
                            "missing-permission",
                            format!(
                                "step '{}' accesses external domain '{}' which is not in declared permissions",
                                step.id, domain
                            ),
                        ));
                    }
                }
            }
        }
    }

    result
}

/// Recursively scan JSON values for unmatched Handlebars template braces.
fn check_template_syntax(
    value: &serde_json::Value,
    step_id: &str,
    result: &mut ValidationResult,
) {
    match value {
        serde_json::Value::String(s) => {
            let opens = s.matches("{{").count();
            let closes = s.matches("}}").count();
            if opens != closes {
                result.diagnostics.push(ValidationDiagnostic::error(
                    "malformed-template",
                    format!(
                        "step '{}': unmatched template braces ({} '{{{{' vs {} '}}}}')",
                        step_id, opens, closes
                    ),
                ));
            }
        }
        serde_json::Value::Object(map) => {
            for v in map.values() {
                check_template_syntax(v, step_id, result);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                check_template_syntax(v, step_id, result);
            }
        }
        _ => {}
    }
}

/// Cross-validate a set of node definitions against their referenced platforms.
///
/// `platform_ids` is the set of platform IDs that have been loaded.
pub fn validate_cross(
    nodes: &[(&NodeDefinition, Option<&std::path::Path>)],
    platform_ids: &HashSet<String>,
) -> ValidationResult {
    let mut result = ValidationResult::default();
    let mut seen_ids: HashSet<String> = HashSet::new();

    for (def, file) in nodes {
        // Rule 8: node's meta.platform must reference a loaded platform
        if let Some(platform) = &def.meta.platform
            && !platform_ids.contains(platform.as_str())
        {
            let mut diag = ValidationDiagnostic::error(
                "unknown-platform-ref",
                format!(
                    "meta.platform '{}' does not match any loaded platform",
                    platform
                ),
            );
            if let Some(p) = file {
                diag = diag.with_file(*p);
            }
            result.diagnostics.push(diag);
        }

        // Rule 9: duplicate meta.id across node files
        if !seen_ids.insert(def.meta.id.clone()) {
            let mut diag = ValidationDiagnostic::error(
                "duplicate-id",
                format!("duplicate node id '{}'", def.meta.id),
            );
            if let Some(p) = file {
                diag = diag.with_file(*p);
            }
            result.diagnostics.push(diag);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::*;
    use serde_json::json;
    use std::collections::HashMap;

    fn make_platform(node_type: &str, base_url: Option<&str>, has_headers: bool) -> PlatformDefinition {
        let mut config = HashMap::new();
        if let Some(url) = base_url {
            config.insert("base_url".into(), json!(url));
        }
        if has_headers {
            config.insert("headers".into(), json!({}));
        }
        PlatformDefinition {
            meta: NodeMeta {
                id: "test_platform".into(),
                name: "Test".into(),
                category: "platform".into(),
                node_type: node_type.into(),
                description: None,
                version: None,
                platform: None,
                data_strategy: None,
                node_subtype: None,
            },
            config,
            settings: vec![],
        }
    }

    fn make_node(
        node_type: &str,
        platform: Option<&str>,
        inputs: Vec<PortDef>,
        outputs: Vec<PortDef>,
        steps: Vec<PipelineStep>,
    ) -> NodeDefinition {
        NodeDefinition {
            meta: NodeMeta {
                id: format!("{}.test.action", platform.unwrap_or("unknown")),
                name: "Test".into(),
                category: "test".into(),
                node_type: node_type.into(),
                description: None,
                version: None,
                platform: platform.map(String::from),
                data_strategy: None,
                node_subtype: None,
            },
            interface: Interface { inputs, outputs, settings: vec![] },
            context: None,
            execution: steps,
            output_transform: None,
            routing: None,
        }
    }

    fn exec_port() -> PortDef {
        PortDef { name: "Exec".into(), data_type: "flow".into(), default_hidden: false, generator: None }
    }

    fn flow_port(name: &str) -> PortDef {
        PortDef { name: name.into(), data_type: "flow".into(), default_hidden: false, generator: None }
    }

    fn http_step(id: &str, url: &str, has_returns: bool) -> PipelineStep {
        let mut returns = HashMap::new();
        if has_returns {
            returns.insert("status".into(), "status_code".into());
            returns.insert("body".into(), "response_body".into());
        }
        PipelineStep {
            id: id.into(),
            tool: "http_client".into(),
            params: json!({ "url": url, "method": "GET" }),
            returns,
        }
    }

    // Platform validation tests

    #[test]
    fn platform_valid() {
        let p = make_platform("Platform", Some("https://api.example.com"), true);
        assert!(validate_platform(&p).is_ok());
    }

    #[test]
    fn platform_wrong_type() {
        let p = make_platform("Action", Some("https://api.example.com"), true);
        let r = validate_platform(&p);
        assert!(r.diagnostics.iter().any(|d| d.rule == "wrong-type"));
    }

    #[test]
    fn platform_missing_base_url() {
        let p = make_platform("Platform", None, true);
        let r = validate_platform(&p);
        assert!(r.diagnostics.iter().any(|d| d.rule == "missing-base-url"));
    }

    #[test]
    fn platform_trailing_slash() {
        let p = make_platform("Platform", Some("https://api.example.com/"), true);
        let r = validate_platform(&p);
        assert!(r.diagnostics.iter().any(|d| d.rule == "trailing-slash-base-url"));
        assert!(!r.has_errors()); // trailing slash is a warning, not error
    }

    #[test]
    fn platform_missing_headers() {
        let p = make_platform("Platform", Some("https://api.example.com"), false);
        let r = validate_platform(&p);
        assert!(r.diagnostics.iter().any(|d| d.rule == "missing-headers"));
    }

    // Node validation tests

    #[test]
    fn node_valid() {
        let node = make_node(
            "Action",
            Some("test_platform"),
            vec![exec_port()],
            vec![flow_port("Success"), flow_port("Error")],
            vec![http_step("call", "{{ platform.base_url }}/endpoint", true)],
        );
        assert!(validate_node(&node).is_ok());
    }

    #[test]
    fn node_missing_exec_port() {
        let node = make_node(
            "Action",
            Some("test_platform"),
            vec![],
            vec![flow_port("Success"), flow_port("Error")],
            vec![],
        );
        let r = validate_node(&node);
        assert!(r.diagnostics.iter().any(|d| d.rule == "missing-exec-port"));
    }

    #[test]
    fn node_wrong_first_port() {
        let bad_port = PortDef {
            name: "Data".into(),
            data_type: "any".into(),
            default_hidden: false,
            generator: None,
        };
        let node = make_node(
            "Action",
            Some("test_platform"),
            vec![bad_port],
            vec![flow_port("Success"), flow_port("Error")],
            vec![],
        );
        let r = validate_node(&node);
        assert!(r.diagnostics.iter().any(|d| d.rule == "missing-exec-port"));
    }

    #[test]
    fn node_missing_success_port() {
        let node = make_node(
            "Action",
            Some("test_platform"),
            vec![exec_port()],
            vec![flow_port("Error")],
            vec![],
        );
        let r = validate_node(&node);
        assert!(r.diagnostics.iter().any(|d| d.rule == "missing-success-port"));
    }

    #[test]
    fn node_missing_error_port() {
        let node = make_node(
            "Action",
            Some("test_platform"),
            vec![exec_port()],
            vec![flow_port("Success")],
            vec![],
        );
        let r = validate_node(&node);
        assert!(r.diagnostics.iter().any(|d| d.rule == "missing-error-port"));
    }

    #[test]
    fn node_http_missing_returns() {
        let node = make_node(
            "Action",
            Some("test_platform"),
            vec![exec_port()],
            vec![flow_port("Success"), flow_port("Error")],
            vec![http_step("call", "{{ platform.base_url }}/ep", false)],
        );
        let r = validate_node(&node);
        assert!(r.diagnostics.iter().any(|d| d.rule == "missing-returns"));
    }

    #[test]
    fn node_hardcoded_url_warning() {
        let node = make_node(
            "Action",
            Some("test_platform"),
            vec![exec_port()],
            vec![flow_port("Success"), flow_port("Error")],
            vec![http_step("call", "https://api.example.com/endpoint", true)],
        );
        let r = validate_node(&node);
        let hardcoded = r.diagnostics.iter().find(|d| d.rule == "hardcoded-url");
        assert!(hardcoded.is_some());
        assert_eq!(hardcoded.unwrap().severity, Severity::Warning);
    }

    #[test]
    fn node_id_not_namespaced_warning() {
        let mut node = make_node(
            "Action",
            Some("myplatform"),
            vec![exec_port()],
            vec![flow_port("Success"), flow_port("Error")],
            vec![],
        );
        node.meta.id = "bad_id".into();
        let r = validate_node(&node);
        assert!(r.diagnostics.iter().any(|d| d.rule == "id-not-namespaced"));
    }

    // Tool name validation tests

    #[test]
    fn node_unknown_tool_warning() {
        let step = PipelineStep {
            id: "bad".into(),
            tool: "httpp_client".into(), // typo
            params: json!({ "url": "{{ platform.base_url }}/ep", "method": "GET" }),
            returns: [("status".into(), "status_code".into()), ("body".into(), "body".into())].into(),
        };
        let node = make_node(
            "Action",
            Some("test_platform"),
            vec![exec_port()],
            vec![flow_port("Success"), flow_port("Error")],
            vec![step],
        );
        let r = validate_node(&node);
        let diag = r.diagnostics.iter().find(|d| d.rule == "unknown-tool");
        assert!(diag.is_some(), "Expected unknown-tool warning");
        assert_eq!(diag.unwrap().severity, Severity::Warning);
    }

    #[test]
    fn node_known_tools_no_warning() {
        let step = http_step("call", "{{ platform.base_url }}/ep", true);
        let node = make_node(
            "Action",
            Some("test_platform"),
            vec![exec_port()],
            vec![flow_port("Success"), flow_port("Error")],
            vec![step],
        );
        let r = validate_node(&node);
        assert!(!r.diagnostics.iter().any(|d| d.rule == "unknown-tool"));
    }

    // Template syntax validation tests

    #[test]
    fn node_malformed_template_missing_close() {
        let step = PipelineStep {
            id: "call".into(),
            tool: "http_client".into(),
            params: json!({ "url": "{{ platform.base_url }/endpoint", "method": "GET" }),
            returns: [("status".into(), "status_code".into()), ("body".into(), "body".into())].into(),
        };
        let node = make_node(
            "Action",
            Some("test_platform"),
            vec![exec_port()],
            vec![flow_port("Success"), flow_port("Error")],
            vec![step],
        );
        let r = validate_node(&node);
        assert!(
            r.diagnostics.iter().any(|d| d.rule == "malformed-template"),
            "Expected malformed-template error"
        );
    }

    #[test]
    fn node_valid_template_no_error() {
        let step = http_step("call", "{{ platform.base_url }}/endpoint", true);
        let node = make_node(
            "Action",
            Some("test_platform"),
            vec![exec_port()],
            vec![flow_port("Success"), flow_port("Error")],
            vec![step],
        );
        let r = validate_node(&node);
        assert!(!r.diagnostics.iter().any(|d| d.rule == "malformed-template"));
    }

    // Cross-validation tests

    #[test]
    fn cross_unknown_platform_ref() {
        let node = make_node(
            "Action",
            Some("missing_platform"),
            vec![exec_port()],
            vec![flow_port("Success"), flow_port("Error")],
            vec![],
        );
        let known = HashSet::new();
        let r = validate_cross(&[(&node, None)], &known);
        assert!(r.diagnostics.iter().any(|d| d.rule == "unknown-platform-ref"));
    }

    #[test]
    fn cross_duplicate_id() {
        let node1 = make_node("Action", Some("p"), vec![exec_port()], vec![flow_port("Success"), flow_port("Error")], vec![]);
        let mut node2 = node1.clone();
        node2.meta.id = node1.meta.id.clone();
        let known: HashSet<String> = ["p".into()].into();
        let r = validate_cross(&[(&node1, None), (&node2, None)], &known);
        assert!(r.diagnostics.iter().any(|d| d.rule == "duplicate-id"));
    }
}
