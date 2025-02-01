use minijinja::{Environment, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

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
        let template_dirs: Vec<PathBuf> = template_dirs.iter().map(PathBuf::from).collect();

        env.set_auto_escape_callback(|name| {
            if name.ends_with(".html") {
                minijinja::AutoEscape::Html
            } else {
                minijinja::AutoEscape::None
            }
        });

        Self::load_templates(&mut env, &template_dirs);

        Self { env }
    }

    /// Load templates into the Environment
    /// ## Args
    /// - env: &mut Environment<'static>
    /// - dirs: &[PathBuf]
    /// ## Returns
    /// - ()
    /// ## Side Effects
    /// - Loads templates into the Environment
    /// ## Panics
    /// - If a template cannot be loaded
    /// - If a directory cannot be read
    fn load_templates(env: &mut Environment<'static>, dirs: &[PathBuf]) {
        for dir in dirs {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();

                    if path.extension().is_some_and(|ext| ext == "html") {
                        if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
                            match fs::read_to_string(&path) {
                                Ok(content) => {
                                    let template_name: &'static str =
                                        Box::leak(file_name.to_string().into_boxed_str());
                                    let arc_content: &'static str =
                                        Box::leak(content.into_boxed_str());

                                    if let Err(err) = env.add_template(template_name, arc_content) {
                                        panic!(
                                            "Error loading template '{}': {}",
                                            path.display(),
                                            err
                                        );
                                    }
                                }
                                Err(err) => {
                                    panic!("Impossible to read file '{}': {}", path.display(), err);
                                }
                            }
                        }
                    }
                }
            } else {
                panic!("Impossible to read directory '{}'", dir.display());
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
    pub(crate) fn render(
        &self,
        template_name: &str,
        context: &HashMap<String, Value>,
    ) -> Result<String, String> {
        self.env
            .get_template(template_name)
            .map_err(|_| format!("Template '{}' not found!", template_name))?
            .render(context)
            .map_err(|e| e.to_string())
    }
}
