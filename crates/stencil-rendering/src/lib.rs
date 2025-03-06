// Copyright (c) 2025 David Stanek <dstanek@dstanek.com>

use regex::Regex;
use std::collections::HashMap;

use minijinja::value::{Kwargs, Rest, Value};
use minijinja::{Environment, State};

mod extract;
mod template_var;

pub use template_var::TemplateVar;

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("Template error: {0}")]
    TemplateError(#[from] minijinja::Error),
    #[error("Error parsing block: {0}")]
    BlockParseError(String),
    #[error("user_content error")]
    FunctionCallError,
}

pub trait Renderable {
    fn content(&self) -> &str;
}

fn user_content_function(
    state: &State,
    args: Rest<Value>,
    kwargs: Kwargs,
) -> Result<String, minijinja::Error> {
    if args.len() > 4 {
        return Err(new_err("user_content: function takes at most 4 arguments"));
    }

    let content_blocks_value = state
        .lookup("content_blocks")
        .ok_or_else(|| new_err("content_blocks not found in environment"))?;
    let content_blocks = content_blocks_value
        .as_object()
        .ok_or_else(|| new_err("content_blocks is not an object"))?
        .downcast_ref::<HashMap<String, String>>()
        .ok_or_else(|| new_err("content_blocks is not a HashMap"))?;

    let key = args
        .first()
        .and_then(|v: &Value| v.as_str())
        .ok_or_else(|| new_err("user_content: first argument must be a string"))
        .unwrap()
        .to_string();
    let begin_comment = get_value(&args, &kwargs, 1, "begin_comment").unwrap_or("#".to_string());
    let end_comment = get_value(&args, &kwargs, 2, "end_comment")
        .map(|s| format!(" {}", s))
        .unwrap_or("".to_string());

    // Use the content from blocks if available, otherwise use default
    let content = match content_blocks.get(&key) {
        Some(content) => content.clone(),
        None => {
            let default = get_value(&args, &kwargs, 3, "default").unwrap_or("".to_string());
            if default.is_empty() {
                format!("{} user content here{}\n", begin_comment, end_comment)
            } else {
                format!("{} {}{}\n", begin_comment, default, end_comment)
            }
        }
    };

    let begin_marker = format!(
        "{} begin-user-content:{}{}\n",
        begin_comment, key, end_comment
    );
    let end_marker = format!(
        "{} end-user-content:{}{}\n",
        begin_comment, key, end_comment
    );

    Ok(format!("{}{}{}", begin_marker, content, end_marker))
}

fn new_err(msg: &str) -> minijinja::Error {
    minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, msg.to_string())
}

/// Transforms `{% include 'file.j2' indent content %}` into `{% filter indent(n, true) %}{% include 'file.j2' %}{% endfilter %}`
fn transform_indent_content(src: &str) -> String {
    let pattern =
        Regex::new(r"(?m)^(?P<indent>\s*)\{\{-?\s*user_content\s*(?P<expr>.+?)\s*-?\}\}").unwrap();

    let result = pattern.replace_all(src, |caps: &regex::Captures| {
        let indent_str = &caps["indent"];
        let include_expr = &caps["expr"];
        let indent_len = indent_str.chars().count();

        format!(
            "{{% filter indent({}, true) %}}{{{{ user_content{} }}}}{{% endfilter %}}",
            indent_len, include_expr,
        )
    });

    result.into_owned()
}

pub fn render<T: Renderable>(
    src: &T,
    dest: &T,
    vars: &HashMap<String, TemplateVar>,
) -> Result<String, RenderError> {
    let mut env = Environment::new();
    let content_blocks = extract::extract_blocks(dest.content())
        .map_err(|e| RenderError::BlockParseError(e.to_string()))?;

    env.set_keep_trailing_newline(true);
    env.add_global("content_blocks", content_blocks);
    env.add_function("user_content", user_content_function);

    let context = vars
        .iter()
        .map(|(k, v)| {
            let v = match v {
                TemplateVar::String(s) => Value::from(s),
                TemplateVar::Int(i) => Value::from(*i),
            };
            (k.to_string(), v)
        })
        .collect::<HashMap<String, Value>>();
    let transformed = transform_indent_content(src.content());
    let tmpl = env.template_from_str(&transformed)?;
    tmpl.render(context).map_err(|e| {
        let detail = e.detail().unwrap_or("").to_string();
        RenderError::TemplateError(e.with_source(new_err(&detail)))
    })
}

pub fn render_str(
    template: &str,
    vars: &HashMap<String, TemplateVar>,
) -> Result<String, RenderError> {
    let mut env = Environment::new();
    env.set_keep_trailing_newline(true);

    let context = vars
        .iter()
        .map(|(k, v)| {
            let v = match v {
                TemplateVar::String(s) => Value::from(s),
                TemplateVar::Int(i) => Value::from(*i),
            };
            (k.to_string(), v)
        })
        .collect::<HashMap<String, Value>>();
    let tmpl = env.template_from_str(template)?;
    tmpl.render(context).map_err(|e| {
        let detail = e.detail().unwrap_or("").to_string();
        RenderError::TemplateError(e.with_source(new_err(&detail)))
    })
}

fn get_value(args: &[Value], kwargs: &Kwargs, index: usize, key: &str) -> Option<String> {
    args.get(index)
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| {
            kwargs
                .get(key)
                .ok()
                .and_then(|v: &Value| v.as_str())
                .map(String::from)
        })
}
