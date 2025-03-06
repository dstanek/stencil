// Copyright (c) 2025 David Stanek <dstanek@dstanek.com>

mod file;
use std::collections::HashMap;

use file::File;

use stencil_rendering::{render, RenderError};

macro_rules! assert_contains {
    ($observed:expr, $expected:expr) => {
        assert!($observed.is_err());
        match $observed {
            Ok(_) => panic!("Expected error, but got Ok"),
            Err(RenderError::TemplateError(e)) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains($expected),
                    "Expected '{}' to contain '{}' at {}:{}",
                    error_msg,
                    $expected,
                    file!(),
                    line!()
                );
            }
            Err(e) => panic!("Expected TemplateError, but got {:?}", e),
        }
    };
}

#[test]
fn test_broken_template() {
    let no_vars = HashMap::new();

    let file = File::new("{{ broken_var }");
    let observed = render(&file, &file, &no_vars);
    assert_contains!(observed, "syntax error");

    let file = File::new("multi-line template\n\n{{ broken_var }");
    let observed = render(&file, &file, &no_vars);
    assert_contains!(observed, "syntax error: unexpected `}`");

    let file = File::new("{% invalid block %}");
    let observed = render(&file, &file, &no_vars);
    assert_contains!(observed, "syntax error: unknown statement");
}

#[test]
fn test_too_many_args() {
    let no_vars = HashMap::new();

    let content = "{{ user_content('main', 'begin', 'end', 'default', 'too many') }}";
    let file = File::new(content);

    let observed = render(&file, &file, &no_vars);
    assert!(observed.is_err());
}
