/// Define an enum that represents the possible value types for templates
#[derive(Debug, Clone)]
pub enum TemplateVar {
    String(String),
    Int(i64),
}

impl From<&str> for TemplateVar {
    fn from(s: &str) -> Self {
        TemplateVar::String(s.to_string())
    }
}

impl From<String> for TemplateVar {
    fn from(s: String) -> Self {
        TemplateVar::String(s)
    }
}

impl From<i64> for TemplateVar {
    fn from(i: i64) -> Self {
        TemplateVar::Int(i)
    }
}

impl From<i32> for TemplateVar {
    fn from(i: i32) -> Self {
        TemplateVar::Int(i as i64)
    }
}
