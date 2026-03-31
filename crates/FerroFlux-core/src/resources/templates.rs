use crate::systems::io::templating::eval_cel;
use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct CelEngine;

impl Default for CelEngine {
    fn default() -> Self {
        Self
    }
}

impl CelEngine {
    pub fn render(
        &self,
        template: &str,
        data: &serde_json::Value,
    ) -> Result<String, String> {
        eval_cel(template, data)
    }
}
