mod detector;
mod normalizer;

pub use detector::*;

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyValueDialect {
    pub total_rows: usize,
    pub field_separator: u8,
}
