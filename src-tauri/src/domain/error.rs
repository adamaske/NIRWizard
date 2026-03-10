use thiserror::Error;

/// Typed errors produced by the SNIRF parser.
///
/// `NoNirsGroup` is called out as its own variant because callers may want to
/// distinguish "file opened fine but wasn't a SNIRF" from a lower-level
/// parse failure. Everything else is captured by `Parse`, which carries the
/// full `anyhow` error chain (including every `.context()` layer added inside
/// the parser).
#[derive(Debug, Error)]
pub enum SnirfError {
    /// The HDF5 file contained no `/nirs` or `/nirs1` group.
    #[error("No /nirs group found in file")]
    NoNirsGroup,

    /// Any other parse failure. Displaying with `{:#}` prints the full chain.
    #[error(transparent)]
    Parse(#[from] anyhow::Error),
}
