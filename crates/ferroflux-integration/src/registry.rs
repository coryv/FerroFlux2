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
                    hooks.on_node_loaded(&def.meta.id, &def);
                    self.definitions.insert(def.meta.id.clone(), def);
                }
                Err(e) => {
                    anyhow::bail!("Failed to parse node definition {:?}: {e}", path);
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
                    anyhow::bail!("Failed to parse platform definition {:?}: {e}", path);
                }
            }
        }

        Ok(result)
    }
}
