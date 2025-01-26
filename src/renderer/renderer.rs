use std::collections::HashMap;
use std::fs;

/// Define the `TemplateRenderer` struct
#[derive(Clone)]
pub struct TemplateRenderer {
    pub template_dirs: Vec<String>,
}

/// Implement the `TemplateRenderer` struct
impl TemplateRenderer {
    /// Create a new `TemplateRenderer` instance
    /// ## Args
    /// - template_dirs: Vec<&str>
    /// ## Returns
    /// - TemplateRenderer
    pub fn new(template_dirs: Vec<&str>) -> Self {
        Self {
            template_dirs: template_dirs.into_iter().map(String::from).collect(),
        }
    }
                             
    /// Render a template with the given context
    /// ## Args
    /// - self
    /// - template_name: &str
    /// - context: &HashMap<&str, &str>
    /// ## Returns
    /// - Result<String, String>
    /// ## Panics
    /// - If the template is not found in any of the configured directories
    pub fn render(&self, template_name: &str, context: &HashMap<&str, &str>) -> Result<String, String> {
        let template_content = self.find_template(template_name)
            .ok_or_else(|| format!("Template '{}' not found in any configured directories!", template_name))?;
    
        let mut rendered = template_content;
        for (key, value) in context {
            let placeholder = format!("{{{{ {} }}}}", key);
            rendered = rendered.replace(&placeholder, value);
        }
    
        Ok(rendered)
    }

    /// Find a template in the configured directories
    /// ## Args
    /// - self
    /// - template_name: &str
    /// ## Returns
    /// - Option<String>
    fn find_template(&self, template_name: &str) -> Option<String> {
        println!("Searching for template '{}' in '{:?}'", template_name, self.template_dirs); 
        for dir in &self.template_dirs {
            let template_path = format!("{}/{}.html", dir, template_name);
            println!("Searching for template '{}' in '{}'", template_name, template_path); 
            if let Ok(content) = fs::read_to_string(&template_path) {
                return Some(content);
            }
        }
        None
    }    
}
