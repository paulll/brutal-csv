mod detector;
mod normalizer;

pub use detector::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct SingleByteDialect {
    pub header: Option<Vec<String>>,

    pub field_separator: u8,
    pub quote_char: Option<u8>,
    pub escape_char: Option<u8>,
    pub empty_columns: Vec<bool>,
    pub numeric_columns: Vec<bool>,
    pub record_terminator: RecordTerminator,

    pub field_separator_is_terminator: bool,
    pub has_escaped_line_breaks: bool,
    pub has_quoted_line_breaks: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum RecordTerminator {
    #[default] Crlf,
    Byte(u8)
}
