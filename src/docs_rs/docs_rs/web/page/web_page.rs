use std::sync::Arc;

use log::info;
use tera::Context;

use super::TemplateData;

use anyhow::Result;

/// adding this to the axum response extensions will lead
/// to the template being rendered, adding the csp_nonce to
/// the context.
pub(crate) struct TemplateRender {
    pub template: String,
    pub context: Context,
}

impl TemplateRender {
    pub fn render_response(
        &self,
        templates: Arc<TemplateData>,
        csp_nonce: String,
    ) -> Result<String> {
        let mut context = self.context.clone();
        context.insert("csp_nonce", &csp_nonce);
        info!("context: {:?}", context);

        let rendered = templates.templates.render(&self.template, &context)?;

        Ok(rendered)
    }
}
