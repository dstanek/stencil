use similar::{ChangeTag, TextDiff};
use std::fmt::Arguments;
use std::io::Write;
use std::path::PathBuf;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, StandardStreamLock, WriteColor};

use crate::error::StencilError;
use crate::output::write;
use crate::source::{Directory, File, Renderable};
use crate::target_config::TargetConfig;

pub fn show_diff(
    changes: &Vec<Renderable>,
    config: &TargetConfig,
    dest: &PathBuf,
) -> Result<(), StencilError> {
    let stdout = StandardStream::stdout(ColorChoice::Always);
    let mut stdout_lock = stdout.lock();

    let stencil_path = PathBuf::from(&config.project.src);
    //let iterator = match file::FilesystemCrawler::new(stencil_path.as_path()).crawl() {
    //    Ok(iterator) => iterator,
    //    Err(e) => {
    //        eprintln!("Error: {}", e);
    //        std::process::exit(1);
    //    }
    //};
    //let iterator = RenderingIterator::new(iterator, &config);
    //let mut ignore = Vec::new();
    // ignore.push(".gitignore".to_string());
    // ignore.push("README.md".to_string());
    // ignore.push("main.tf".to_string());
    // ignore.push("outputs.tf".to_string());
    // ignore.push("terraform.tf".to_string());
    // ignore.push("variables.tf".to_string());

    for entry in changes {
        match entry {
            Renderable::File(file) => {
                // println!("File: {:?}", file.relative_path);
                let orig_filename = dest.join(&file.relative_path);
                let orig_file = match orig_filename.exists() {
                    true => File::from_path(file.relative_path.clone(), &orig_filename).unwrap(),
                    false => File::empty(),
                };
                // println!("File: {:?} {:?}", file.relative_path, file.content);
                show_file_diff(&mut stdout_lock, &orig_file, &file)?;
            }
            Renderable::Directory(dir) => {
                // println!("Directory: {:?}", dir.relative_path);
                if !dest.join(&dir.relative_path).exists() {
                    show_directory_diff(&mut stdout_lock, &dir)?;
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
    if old_path != std::path::Path::new("/dev/null") && old_content == new_content {
        return Ok(());
    }
    // Compute the diff between the two files
    let diff = TextDiff::from_lines(old_content, new_content);

    // Print the diff header
    write(
        &mut handle,
        Color::Yellow,
        format!(
            "diff --git a/{} b/{}\n",
            new_path.display(),
            new_path.display()
        ),
    )?;
    if old_path != std::path::Path::new("/dev/null") {
        write(
            &mut handle,
            Color::Blue,
            format!("--- old/{}\n", old_path.display()),
        )?;
    } else {
        write(
            &mut handle,
            Color::Blue,
            format!("--- old/{}    (file not found)\n", new_path.display()),
        )?;
    }
    if new_content.is_empty() {
        write!(
            &mut handle,
            "+++ new/{}    (new empty file)\n",
            new_path.display()
        )?;
    } else {
        write!(&mut handle, "+++ new/{}\n", new_path.display())?;
    }

    // Iterate over the diff hunks
    for (_, group) in diff.grouped_ops(3).iter().enumerate() {
        // Print hunk header
        let (old_start, old_end) = {
            let old_indices = group.iter().filter_map(|op| {
                if op.old_range().len() > 0 {
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
                if op.new_range().len() > 0 {
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
        write!(
            &mut handle,
            "@@ -{},{} +{},{} @@\n",
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
    let rp = dir.relative_path.display();
    write(
        &mut handle,
        Color::Yellow,
        format!("diff --git a/{} b/{}\n", rp, rp,),
    )?;
    write(
        &mut handle,
        Color::Blue,
        format!("--- old/{}    (directory not found)\n", rp),
    )?;
    write(
        &mut handle,
        Color::White,
        format!("+++ new/{}    (new directory)\n\n", rp),
    )?;
    Ok(())
}

fn write_color(
    handle: &mut StandardStreamLock,
    color: Color,
    args: Arguments,
) -> Result<(), StencilError> {
    handle.set_color(ColorSpec::new().set_fg(Some(color)))?;
    write!(handle, "{}", args)?;
    handle.reset()?;
    Ok(())
}
