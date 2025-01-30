use std::collections::HashMap;
use std::fs;
use minijinja::{Environment, Value};

/// Define the Context type
pub type Context = HashMap<String, Value>;

/// Define the TemplateRenderer struct
/// ## Fields
/// - env: Environment<'static>
#[derive(Clone)]
pub(crate) struct TemplateRenderer {
    env: Environment<'static>,
}

/// Implement the TemplateRenderer struct
impl TemplateRenderer {

    /// Create a new TemplateRenderer instance
    /// ## Args
    /// - template_dirs: Vec<&str>
    /// ## Returns
    /// - TemplateRenderer
    pub(crate) fn new(template_dirs: Vec<&str>) -> Self {
        let mut env = Environment::new();
        let template_dirs: Vec<String> = template_dirs.iter().map(|s| s.to_string()).collect();
        Self::load_templates(&mut env, &template_dirs);

        Self { env }
    }

    /// Load templates into the Environment
    /// ## Args
    /// - env: &mut Environment<'static>
    /// - dirs: &[String]
    /// ## Returns
    /// - ()
    /// ## Side Effects
    /// - Loads templates into the Environment
    fn load_templates(env: &mut Environment<'static>, dirs: &[String]) {
        for dir in dirs {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "html") {
                        if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
                            if let Ok(content) = fs::read_to_string(&path) {
                                let template_name: &'static str = Box::leak(file_name.to_string().into_boxed_str());
                                let arc_content: &'static str = Box::leak(content.into_boxed_str());
    
                                env.add_template(template_name, arc_content).unwrap();
                            }
                        }
                    }
                }
            }
        }
    }

    /// Render a template with the given context
    /// ## Args
    /// - self
    /// - template_name: &str
    /// - context: &HashMap<String, Value>
    /// ## Returns
    /// - Result<String, String>
    pub(crate) fn render(&self, template_name: &str, context: &HashMap<String, Value>) -> Result<String, String> {
        self.env
            .get_template(template_name)
            .map_err(|_| format!("Template '{}' not found!", template_name))?
            .render(context)
            .map_err(|e| e.to_string())
    }
}
