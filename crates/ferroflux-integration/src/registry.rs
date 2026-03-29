use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::definition::{NodeDefinition, PlatformDefinition};
use crate::hooks::{LoadHooks, NoopHooks};
use crate::validation::{validate_cross, validate_node, validate_platform, ValidationResult};

/// Holds all loaded platform and node definitions.
///
/// This is a pure data store — no runtime or ECS dependencies.
/// Wrap it in a Bevy `Resource` in `ferroflux_core` if needed.
#[derive(Debug, Default, Clone)]
pub struct DefinitionRegistry {
    pub definitions: HashMap<String, NodeDefinition>,
    pub platforms: HashMap<String, PlatformDefinition>,
}

impl DefinitionRegistry {
    pub fn clear(&mut self) {
        self.definitions.clear();
        self.platforms.clear();
    }

    /// Load all `.yaml` / `.yml` files from a directory tree, validate, and
    /// return a combined `ValidationResult`. Fires `NoopHooks`.
    pub fn load_from_dir(&mut self, path: &Path) -> anyhow::Result<ValidationResult> {
        self.load_from_dir_with_hooks(path, &mut NoopHooks)
    }

    /// Like `load_from_dir` but fires `hooks` callbacks for each loaded definition.
    pub fn load_from_dir_with_hooks(
        &mut self,
        path: &Path,
        hooks: &mut dyn LoadHooks,
    ) -> anyhow::Result<ValidationResult> {
        let mut result = ValidationResult::default();

        if path.is_dir() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let child = entry.path();
                if child.is_dir() {
                    let sub = self.load_from_dir_with_hooks(&child, hooks)?;
                    result.merge(sub);
                } else if child
                    .extension()
                    .is_some_and(|ext| ext == "yaml" || ext == "yml")
                {
                    let file_result = self.load_file_with_hooks(&child, hooks)?;
                    result.merge(file_result);
                }
            }
        }

        // Cross-validation once all files are loaded
        let platform_ids: HashSet<String> = self.platforms.keys().cloned().collect();
        let nodes: Vec<_> = self.definitions.values().map(|n| (n, None)).collect();
        let cross = validate_cross(&nodes, &platform_ids);
        for diag in &cross.diagnostics {
            hooks.on_validation_error(diag);
        }
        result.merge(cross);

        Ok(result)
    }

    fn load_file_with_hooks(
        &mut self,
        path: &Path,
        hooks: &mut dyn LoadHooks,
    ) -> anyhow::Result<ValidationResult> {
        let mut result = ValidationResult::default();
        let content = std::fs::read_to_string(path)?;

        // Discriminate platform vs node by presence of the `execution:` key.
        // Platform files don't have execution pipelines.
        if content.contains("execution:") {
            match serde_yaml::from_str::<NodeDefinition>(&content) {
                Ok(def) => {
                    let mut diags = validate_node(&def);
                    for d in &diags.diagnostics {
                        hooks.on_validation_error(d);
                    }
                    // Attach file path to diagnostics
                    for d in &mut diags.diagnostics {
                        d.file = Some(path.to_path_buf());
                    }
                    result.merge(diags);
                    // Detect duplicate IDs before inserting
                    if self.definitions.contains_key(&def.meta.id) {
                        let diag = crate::validation::ValidationDiagnostic {
                            severity: crate::validation::Severity::Error,
                            rule: "duplicate-id",
                            message: format!("duplicate node id '{}'", def.meta.id),
                            file: Some(path.to_path_buf()),
                        };
                        hooks.on_validation_error(&diag);
                        result.diagnostics.push(diag);
                    } else {
                        hooks.on_node_loaded(&def.meta.id, &def);
                        self.definitions.insert(def.meta.id.clone(), def);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to parse node definition {:?}: {e}", path);
                }
            }
        } else {
            match serde_yaml::from_str::<PlatformDefinition>(&content) {
                Ok(def) => {
                    let mut diags = validate_platform(&def);
                    for d in &diags.diagnostics {
                        hooks.on_validation_error(d);
                    }
                    for d in &mut diags.diagnostics {
                        d.file = Some(path.to_path_buf());
                    }
                    result.merge(diags);
                    hooks.on_platform_loaded(&def.meta.id, &def);
                    self.platforms.insert(def.meta.id.clone(), def);
                }
                Err(e) => {
                    tracing::error!("Failed to parse platform definition {:?}: {e}", path);
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Write `content` to `dir/filename` and return the full path.
    fn write_file(dir: &std::path::Path, filename: &str, content: &str) -> std::path::PathBuf {
        let path = dir.join(filename);
        fs::write(&path, content).unwrap();
        path
    }

    const VALID_PLATFORM_YAML: &str = r#"
meta:
  id: test_platform
  name: Test Platform
  category: test
  type: Platform
config:
  base_url: "https://api.example.com"
  headers: {}
settings: []
"#;

    const VALID_NODE_YAML: &str = r#"
meta:
  id: test_platform.action.do_thing
  name: Do Thing
  category: test
  type: Action
  platform: test_platform
interface:
  inputs:
    - name: Exec
      type: flow
  outputs:
    - name: Success
      type: flow
    - name: Error
      type: flow
  settings: []
execution:
  - id: call
    tool: http_client
    params:
      url: "{{ platform.base_url }}/things"
      method: GET
    returns:
      status: status_code
      body: response_body
"#;

    // ── load_from_dir: happy path ──────────────────────────────────────────────

    #[test]
    fn load_valid_platform_and_node() {
        let tmp = tempdir();
        write_file(&tmp, "platform.yaml", VALID_PLATFORM_YAML);
        write_file(&tmp, "node.yaml", VALID_NODE_YAML);

        let mut reg = DefinitionRegistry::default();
        let result = reg.load_from_dir(&tmp).unwrap();

        assert!(result.is_ok(), "Expected no errors, got: {:?}", result.diagnostics);
        assert!(reg.platforms.contains_key("test_platform"));
        assert!(reg.definitions.contains_key("test_platform.action.do_thing"));
    }

    #[test]
    fn load_from_subdirectory_recursively() {
        let tmp = tempdir();
        let sub = tmp.join("sub");
        fs::create_dir(&sub).unwrap();
        write_file(&tmp, "platform.yaml", VALID_PLATFORM_YAML);
        write_file(&sub, "node.yaml", VALID_NODE_YAML);

        let mut reg = DefinitionRegistry::default();
        reg.load_from_dir(&tmp).unwrap();

        assert!(reg.platforms.contains_key("test_platform"));
        assert!(reg.definitions.contains_key("test_platform.action.do_thing"));
    }

    #[test]
    fn non_yaml_files_are_ignored() {
        let tmp = tempdir();
        write_file(&tmp, "README.md", "# docs");
        write_file(&tmp, "platform.yaml", VALID_PLATFORM_YAML);

        let mut reg = DefinitionRegistry::default();
        reg.load_from_dir(&tmp).unwrap();

        assert_eq!(reg.platforms.len(), 1);
        assert_eq!(reg.definitions.len(), 0);
    }

    // ── cross-validation errors surface from load_from_dir ───────────────────

    #[test]
    fn cross_validation_unknown_platform_surfaces_as_error() {
        let tmp = tempdir();
        // Node references "missing_platform" which is never loaded
        let node_yaml = r#"
meta:
  id: missing_platform.action.do_thing
  name: Do Thing
  category: test
  type: Action
  platform: missing_platform
interface:
  inputs:
    - name: Exec
      type: flow
  outputs:
    - name: Success
      type: flow
    - name: Error
      type: flow
  settings: []
execution:
  - id: call
    tool: http_client
    params:
      url: "{{ platform.base_url }}/ep"
      method: GET
    returns:
      status: status_code
      body: response_body
"#;
        write_file(&tmp, "node.yaml", node_yaml);

        let mut reg = DefinitionRegistry::default();
        let result = reg.load_from_dir(&tmp).unwrap();

        assert!(
            result.diagnostics.iter().any(|d| d.rule == "unknown-platform-ref"),
            "Expected unknown-platform-ref diagnostic"
        );
        assert!(result.has_errors());
    }

    #[test]
    fn cross_validation_duplicate_id() {
        let tmp = tempdir();
        let sub = tmp.join("sub");
        fs::create_dir(&sub).unwrap();
        write_file(&tmp, "platform.yaml", VALID_PLATFORM_YAML);
        write_file(&tmp, "node1.yaml", VALID_NODE_YAML);
        write_file(&sub, "node2.yaml", VALID_NODE_YAML); // same id

        let mut reg = DefinitionRegistry::default();
        let result = reg.load_from_dir(&tmp).unwrap();

        assert!(
            result.diagnostics.iter().any(|d| d.rule == "duplicate-id"),
            "Expected duplicate-id diagnostic"
        );
    }

    // ── hooks fire correctly ─────────────────────────────────────────────────

    #[test]
    fn hooks_called_for_loaded_definitions() {
        use crate::hooks::LoadHooks;
        use crate::validation::ValidationDiagnostic;

        struct TrackingHooks {
            nodes_loaded: Vec<String>,
            platforms_loaded: Vec<String>,
        }
        impl LoadHooks for TrackingHooks {
            fn on_node_loaded(&mut self, id: &str, _def: &NodeDefinition) {
                self.nodes_loaded.push(id.to_string());
            }
            fn on_platform_loaded(&mut self, id: &str, _def: &PlatformDefinition) {
                self.platforms_loaded.push(id.to_string());
            }
            fn on_validation_error(&mut self, _diag: &ValidationDiagnostic) {}
        }

        let tmp = tempdir();
        write_file(&tmp, "platform.yaml", VALID_PLATFORM_YAML);
        write_file(&tmp, "node.yaml", VALID_NODE_YAML);

        let mut hooks = TrackingHooks { nodes_loaded: vec![], platforms_loaded: vec![] };
        let mut reg = DefinitionRegistry::default();
        reg.load_from_dir_with_hooks(&tmp, &mut hooks).unwrap();

        assert_eq!(hooks.platforms_loaded, vec!["test_platform"]);
        assert_eq!(hooks.nodes_loaded, vec!["test_platform.action.do_thing"]);
    }

    // ── invalid YAML returns an error ────────────────────────────────────────

    #[test]
    fn invalid_yaml_returns_error() {
        let tmp = tempdir();
        // `execution:` key makes it attempt node parse, but YAML is malformed
        write_file(&tmp, "bad.yaml", "execution: [[[broken");

        let mut reg = DefinitionRegistry::default();
        let result = reg.load_from_dir(&tmp);
        assert!(result.is_err(), "Should return an error for invalid YAML");
    }

    // ── clear wipes definitions ───────────────────────────────────────────────

    #[test]
    fn clear_removes_all_definitions() {
        let tmp = tempdir();
        write_file(&tmp, "platform.yaml", VALID_PLATFORM_YAML);

        let mut reg = DefinitionRegistry::default();
        reg.load_from_dir(&tmp).unwrap();
        assert!(!reg.platforms.is_empty());

        reg.clear();
        assert!(reg.platforms.is_empty());
        assert!(reg.definitions.is_empty());
    }

    // ── helper: create a unique temp directory ───────────────────────────────

    fn tempdir() -> std::path::PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let path = std::env::temp_dir().join(format!("ff_reg_{pid}_{n}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
