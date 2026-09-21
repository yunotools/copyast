use super::catalog;

pub fn generate() -> String {
    render(catalog::list_templates())
}

pub(crate) fn render<'a>(
    templates: impl IntoIterator<Item = &'a catalog::TemplateDefinition>,
) -> String {
    let mut output = String::new();

    for (template_index, template) in templates.into_iter().enumerate() {
        if template_index > 0 {
            output.push('\n');
        }

        output.push_str("# =========================\n# ");
        output.push_str(template.title);
        output.push_str("\n# =========================\n");
        output.push_str(template.content.trim());
        output.push('\n');
    }

    output
}
