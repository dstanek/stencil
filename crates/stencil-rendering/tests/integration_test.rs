// Copyright (c) 2025 David Stanek <dstanek@dstanek.com>

mod file;
use std::collections::HashMap;

use file::File;

use textwrap::dedent;

use stencil_rendering::{render, render_str, TemplateVar};

#[test]
fn test_render_python() {
    let content = dedent(
        r#"
            import sys

            def main(args):
                {{ user_content("main", '#') }}

            if __name__ == "__main__":
                main(sys.argv[1:])
            "#,
    );
    let file = File::new(&content);

    let expected = dedent(
        r#"
            import sys

            def main(args):
                # begin-user-content:main
                # user content here
                # end-user-content:main

            if __name__ == "__main__":
                main(sys.argv[1:])
        "#,
    );

    let observed = render(&file, &file, &HashMap::new()).unwrap();
    assert_eq!(observed.trim(), expected.trim());
}

#[test]
fn test_render_js_prefix_only() {
    let content = dedent(
        r#"
            function f() {
                {{ user_content("f", "//") }}
            }
        "#,
    );
    let file = File::new(&content);

    let expected = dedent(
        r#"
            function f() {
                // begin-user-content:f
                // user content here
                // end-user-content:f
            }
        "#,
    );

    let observed = render(&file, &file, &HashMap::new()).unwrap();
    assert_eq!(observed.trim(), expected.trim());
}

#[test]
fn test_render_html() {
    let content = dedent(
        r#"
            <html>
                <body>
                    {{ user_content("body", "<!--", "-->") }}
                </body>
            </html>
            "#,
    );
    let file = File::new(&content);

    let expected = dedent(
        r#"
            <html>
                <body>
                    <!-- begin-user-content:body -->
                    <!-- user content here -->
                    <!-- end-user-content:body -->
                </body>
            </html>
        "#,
    );

    let observed = render(&file, &file, &HashMap::new()).unwrap();
    assert_eq!(observed.trim(), expected.trim());
}

#[test]
fn test_render_python_custom_default() {
    let content = dedent(
        r#"
            import sys

            def main(args):
                {{ user_content("main", '#', None, "your code here") }}

            if __name__ == "__main__":
                main(sys.argv[1:])
            "#,
    );
    let file = File::new(&content);

    let expected = dedent(
        r#"
            import sys

            def main(args):
                # begin-user-content:main
                # your code here
                # end-user-content:main

            if __name__ == "__main__":
                main(sys.argv[1:])
        "#,
    );

    let observed = render(&file, &file, &HashMap::new()).unwrap();
    assert_eq!(observed.trim(), expected.trim());
}

#[test]
fn test_render_python_custom_default_kwargs() {
    let content = dedent(
        r#"
            import sys

            def main(args):
                {{ user_content("main", '#', default="your code here") }}

            if __name__ == "__main__":
                main(sys.argv[1:])
            "#,
    );
    let file = File::new(&content);

    let expected = dedent(
        r#"
            import sys

            def main(args):
                # begin-user-content:main
                # your code here
                # end-user-content:main

            if __name__ == "__main__":
                main(sys.argv[1:])
        "#,
    );

    let observed = render(&file, &file, &HashMap::new()).unwrap();
    assert_eq!(observed.trim(), expected.trim());
}

#[test]
fn test_simple_rendering() {
    let template_file = File::new(
        dedent(
            r#"
            {{ shape }}
            {%- for i in range(size) %}
            {{ i }}
            {%- endfor %}
        "#,
        )
        .as_str(),
    );

    let expected = dedent(
        r#"
            square
            0
            1
            2
        "#,
    );

    let vars = HashMap::from([
        ("shape".to_string(), TemplateVar::from("square")),
        ("size".to_string(), TemplateVar::Int(3)),
    ]);
    let observed = render(&template_file, &template_file, &vars).unwrap();
    assert_eq!(observed.trim(), expected.trim());
}

#[test]
fn test_simple_string_rendering() {
    let template = dedent(
        r#"
            {{ shape }}
            {%- for i in range(size) %}
            {{ i }}
            {%- endfor %}
        "#,
    );

    let expected = dedent(
        r#"
            square
            0
            1
            2
        "#,
    );

    let vars = HashMap::from([
        ("shape".to_string(), TemplateVar::from("square")),
        ("size".to_string(), TemplateVar::Int(3)),
    ]);
    let observed = render_str(&template, &vars).unwrap();
    assert_eq!(observed.trim(), expected.trim());
}

#[test]
fn test_render_python_with_existing_content() {
    let existing_content = dedent(
        r#"
            import sys

            def main(args):
                # begin-user-content:main
                print("I found args", args)
                print("This is existing content")
                # end-user-content:main

            if __name__ == "__main__":
                main(sys.argv[1:])
        "#,
    );
    let existing_file = File::new(&existing_content);

    let updated_template = dedent(
        r#"
            import sys

            def main(args) -> int:
                {{ user_content("main", '#', None, "your code here") }}

                return 0

            if __name__ == "__main__":
                sys.exit(main(sys.argv[1:]))
            "#,
    );
    let updated_template_file = File::new(&updated_template);

    let expected = dedent(
        r#"
            import sys

            def main(args) -> int:
                # begin-user-content:main
                print("I found args", args)
                print("This is existing content")
                # end-user-content:main

                return 0

            if __name__ == "__main__":
                sys.exit(main(sys.argv[1:]))
        "#,
    );

    let observed = match render(&updated_template_file, &existing_file, &HashMap::new()) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error during rendering: {:?}", e);
            panic!("Rendering failed");
        }
    };
    assert_eq!(observed.trim(), expected.trim());
}
