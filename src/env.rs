pub const COMPACT_WINDOW: &str = "CLAUDE_CODE_AUTO_COMPACT_WINDOW";

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("{COMPACT_WINDOW} is not unicode: {0:?}")]
    NotUnicode(std::ffi::OsString),

    #[error("Invalid {COMPACT_WINDOW}")]
    ParseCompactWindow(#[source] std::num::ParseIntError),
}

pub fn compact_window() -> Result<Option<std::num::NonZeroU64>, Error> {
    let Some(raw) = std::env::var_os(COMPACT_WINDOW) else {
        return Ok(None);
    };
    let text = raw.into_string().map_err(Error::NotUnicode)?;
    let limit = text.parse().map_err(Error::ParseCompactWindow)?;
    let window = Some(limit);
    Ok(window)
}
