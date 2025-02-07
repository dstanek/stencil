// Copyright 2024-2025 David Stanek <dstanek@dstanek.com>

use similar::{ChangeTag, TextDiff};
use std::io::Write;
use std::path::Path;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, StandardStreamLock, WriteColor};

use crate::output::write;
use crate::target_config::TargetConfig;
use stencil_error::StencilError;
use stencil_source::{Directory, File, Renderable};

pub fn show_diff(
    changes: &Vec<Renderable>,
    _config: &TargetConfig,
    dest: &Path,
) -> Result<(), StencilError> {
    let stdout = StandardStream::stdout(ColorChoice::Always);
    let mut stdout_lock = stdout.lock();

    // TODO: implement a way to ignore certain files
    // let mut ignore = Vec::new();
    // ignore.push("README.md".to_string());

    for entry in changes {
        match entry {
            Renderable::File(file) => {
                let orig_filename = dest.join(&file.relative_path);
                let orig_file = match orig_filename.exists() {
                    true => File::from_path(file.relative_path.clone(), &orig_filename).unwrap(),
                    false => File::empty(),
                };
                show_file_diff(&mut stdout_lock, &orig_file, file)?;
            }
            Renderable::Directory(dir) => {
                if !dest.join(&dir.relative_path).exists() {
                    show_directory_diff(&mut stdout_lock, dir)?;
                };
            }
        }
    }
    Ok(())
}

fn show_file_diff(
    mut handle: &mut StandardStreamLock,
    old: &File,
    new: &File,
) -> Result<(), StencilError> {
    let old_path = &old.relative_path;
    let new_path = &new.relative_path;
    let old_content = old.content.as_str();
    let new_content = new.content.as_str();

    // Files are identical so no need to show a diff
    if old_path != "/dev/null" && old_content == new_content {
        return Ok(());
    }
    // Compute the diff between the two files
    let diff = TextDiff::from_lines(old_content, new_content);

    // Print the diff header
    write(
        &mut handle,
        Color::Yellow,
        format!("diff --git a/{} b/{}\n", new_path, new_path),
    )?;
    if old_path != "/dev/null" {
        write(&mut handle, Color::Blue, format!("--- old/{}\n", old_path))?;
    } else {
        write(
            &mut handle,
            Color::Blue,
            format!("--- old/{}    (file not found)\n", new_path),
        )?;
    }
    if new_content.is_empty() {
        writeln!(&mut handle, "+++ new/{}    (new empty file)", new_path)?;
    } else {
        writeln!(&mut handle, "+++ new/{}", new_path)?;
    }

    // Iterate over the diff hunks
    for group in diff.grouped_ops(3).iter() {
        // Print hunk header
        let (old_start, old_end) = {
            let old_indices = group.iter().filter_map(|op| {
                if op.old_range().is_empty() {
                    Some(op.old_range().start + 1)
                } else {
                    None
                }
            });
            let start = old_indices.clone().min().unwrap_or(0);
            let end = old_indices.max().unwrap_or(0);
            (start, end)
        };
        let (new_start, new_end) = {
            let new_indices = group.iter().filter_map(|op| {
                if op.new_range().is_empty() {
                    Some(op.new_range().start + 1)
                } else {
                    None
                }
            });
            let start = new_indices.clone().min().unwrap_or(0);
            let end = new_indices.max().unwrap_or(0);
            (start, end)
        };

        handle.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))?;
        writeln!(
            &mut handle,
            "@@ -{},{} +{},{} @@",
            old_start,
            old_end - old_start + 1,
            new_start,
            new_end - new_start + 1
        )?;
        handle.reset()?;

        // Iterate over the changes in the hunk
        for op in group {
            for change in diff.iter_changes(op) {
                match change.tag() {
                    ChangeTag::Delete => {
                        handle.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
                        write!(
                            &mut handle,
                            "-{:4} {}",
                            change.old_index().map(|i| i + 1).unwrap_or(0),
                            change
                        )?;
                    }
                    ChangeTag::Insert => {
                        handle.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
                        write!(
                            &mut handle,
                            "+{:4} {}",
                            change.new_index().map(|i| i + 1).unwrap_or(0),
                            change
                        )?;
                    }
                    ChangeTag::Equal => {
                        handle.set_color(ColorSpec::new().set_fg(None))?;
                        write!(
                            &mut handle,
                            " {:4} {}",
                            change.old_index().map(|i| i + 1).unwrap_or(0),
                            change
                        )?;
                    }
                }
            }
        }
        handle.reset()?;
    }
    println!();
    Ok(())
}

fn show_directory_diff(
    mut handle: &mut StandardStreamLock,
    dir: &Directory,
) -> Result<(), StencilError> {
    write(
        &mut handle,
        Color::Yellow,
        format!(
            "diff --git a/{} b/{}\n",
            dir.relative_path, dir.relative_path,
        ),
    )?;
    write(
        &mut handle,
        Color::Blue,
        format!("--- old/{}    (directory not found)\n", dir.relative_path),
    )?;
    write(
        &mut handle,
        Color::White,
        format!("+++ new/{}    (new directory)\n\n", dir.relative_path),
    )?;
    Ok(())
}
