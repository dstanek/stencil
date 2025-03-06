// Copyright (c) 2024-2025 David Stanek <dstanek@dstanek.com>

use termcolor::{Color, ColorSpec, WriteColor};

use stencil_error::StencilError;

pub fn write(stream: &mut dyn WriteColor, color: Color, msg: &str) -> Result<(), StencilError> {
    stream.set_color(ColorSpec::new().set_fg(Some(color)))?;
    write!(stream, "{msg}")?;
    stream.reset()?;
    Ok(())
}

pub fn write_bold(
    stream: &mut dyn WriteColor,
    color: Color,
    msg: &str,
) -> Result<(), StencilError> {
    stream.set_color(ColorSpec::new().set_fg(Some(color)).set_bold(true))?;
    write!(stream, "{msg}")?;
    stream.reset()?;
    Ok(())
}
