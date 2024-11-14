use termcolor::{Color, ColorSpec, WriteColor};

use crate::error::StencilError;

pub fn write(stream: &mut dyn WriteColor, color: Color, msg: String) -> Result<(), StencilError> {
    stream.set_color(ColorSpec::new().set_fg(Some(color)))?;
    write!(stream, "{}", msg)?;
    stream.reset()?;
    Ok(())
}

pub fn write_bold(
    stream: &mut dyn WriteColor,
    color: Color,
    msg: String,
) -> Result<(), StencilError> {
    stream.set_color(ColorSpec::new().set_fg(Some(color)).set_bold(true))?;
    write!(stream, "{}", msg)?;
    stream.reset()?;
    Ok(())
}

// fn write_color(
//     handle: &mut StandardStreamLock,
//     color: Color,
//     args: Arguments,
// ) -> Result<(), StencilError> {
//     handle.set_color(ColorSpec::new().set_fg(Some(color)))?;
//     write!(handle, "{}", args)?;
//     handle.reset()?;
//     Ok(())
// }
