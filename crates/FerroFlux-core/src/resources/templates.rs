use bevy_ecs::prelude::*;
use handlebars::{Context, Handlebars, HelperResult, Output, RenderContext};

#[derive(Resource)]
pub struct TemplateEngine {
    pub hbs: Handlebars<'static>,
}

impl Default for TemplateEngine {
    fn default() -> Self {
        let mut hbs = Handlebars::new();
        hbs.set_strict_mode(false);

        // Register helpers
        hbs.register_helper(
            "json",
            Box::new(
                |h: &handlebars::Helper,
                 _: &Handlebars,
                 _: &Context,
                 _: &mut RenderContext,
                 out: &mut dyn Output|
                 -> HelperResult {
                    let param = h.param(0).ok_or(handlebars::RenderErrorReason::Other(
                        "Param 0 is required for json helper".to_string(),
                    ))?;
                    let value = param.value();
                    out.write(&serde_json::to_string(value).map_err(|e| {
                        handlebars::RenderErrorReason::Other(format!("JSON encode failed: {}", e))
                    })?)?;
                    Ok(())
                },
            ),
        );

        hbs.register_helper(
            "eq",
            Box::new(
                |h: &handlebars::Helper,
                 _: &Handlebars,
                 _: &Context,
                 _: &mut RenderContext,
                 out: &mut dyn Output|
                 -> HelperResult {
                    let p1 = h.param(0).ok_or(handlebars::RenderErrorReason::Other(
                        "Param 0 required".to_string(),
                    ))?;
                    let p2 = h.param(1).ok_or(handlebars::RenderErrorReason::Other(
                        "Param 1 required".to_string(),
                    ))?;
                    if p1.value() == p2.value() {
                        out.write("true")?;
                    }
                    Ok(())
                },
            ),
        );

        hbs.register_helper(
            "is_string",
            Box::new(
                |h: &handlebars::Helper,
                 _: &Handlebars,
                 _: &Context,
                 _: &mut RenderContext,
                 out: &mut dyn Output|
                 -> HelperResult {
                    let param = h.param(0).ok_or(handlebars::RenderErrorReason::Other(
                        "Param 0 required".to_string(),
                    ))?;
                    if param.value().is_string() {
                        out.write("true")?;
                    }
                    Ok(())
                },
            ),
        );

        hbs.register_helper(
            "is_array",
            Box::new(
                |h: &handlebars::Helper,
                 _: &Handlebars,
                 _: &Context,
                 _: &mut RenderContext,
                 out: &mut dyn Output|
                 -> HelperResult {
                    let param = h.param(0).ok_or(handlebars::RenderErrorReason::Other(
                        "Param 0 required".to_string(),
                    ))?;
                    if param.value().is_array() {
                        out.write("true")?;
                    }
                    Ok(())
                },
            ),
        );

        Self { hbs }
    }
}

impl TemplateEngine {
    pub fn render(
        &self,
        template: &str,
        data: &serde_json::Value,
    ) -> Result<String, handlebars::RenderError> {
        self.hbs.render_template(template, data)
    }
}
