mod detector;
mod normalizer;

use std::cmp::Ordering;
pub use detector::*;

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
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

    pub total_rows: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum RecordTerminator {
    #[default] Crlf,
    Byte(u8)
}

impl PartialOrd<Self> for SingleByteDialect {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // header is preferred over no-header
        if self.header.is_some() && other.header.is_none() {
            return Some(Ordering::Greater)
        }
        if self.header.is_none() && other.header.is_some() {
            return Some(Ordering::Less)
        }

        // field_separator_is_terminator is preferred
        if self.field_separator_is_terminator && !other.field_separator_is_terminator {
            return Some(Ordering::Greater)
        }
        if !self.field_separator_is_terminator && other.field_separator_is_terminator {
            return Some(Ordering::Less)
        }

        // more numeric columns is preferred
        let numeric_self = self.numeric_columns
            .iter()
            .zip(self.empty_columns.iter())
            .filter(|(is_numeric, is_empty)| **is_numeric && !**is_empty)
            .count();
        let numeric_other = other.numeric_columns
            .iter()
            .zip(other.empty_columns.iter())
            .filter(|(is_numeric, is_empty)| **is_numeric && !**is_empty)
            .count();
        if numeric_self > numeric_other {
            return Some(Ordering::Greater)
        }
        if numeric_self < numeric_other {
            return Some(Ordering::Less)
        }

        // has_escaped_line_breaks and has_quoted_line_breaks are not preferred
        // because most cases when has_escaped_line_breaks=true is valid then =false is valid too
        // and same for has_quoted_line_breaks
        if !self.has_escaped_line_breaks && other.has_escaped_line_breaks {
            return Some(Ordering::Greater)
        }
        if self.has_escaped_line_breaks && !other.has_escaped_line_breaks {
            return Some(Ordering::Less)
        }
        if !self.has_quoted_line_breaks && other.has_quoted_line_breaks {
            return Some(Ordering::Greater)
        }
        if self.has_quoted_line_breaks && !other.has_quoted_line_breaks {
            return Some(Ordering::Less)
        }

        // pessimize too long headers (100+ unicode characters)
        let has_long_header_self = self.header
            .iter()
            .flatten()
            .any(|x| x.chars().count() > 100);
        let has_long_header_other = other.header
            .iter()
            .flatten()
            .any(|x| x.chars().count() > 100);
        if !has_long_header_self && has_long_header_other {
            return Some(Ordering::Greater)
        }
        if has_long_header_self && !has_long_header_other {
            return Some(Ordering::Less)
        }

        // more rows is preferred
        let rows_difference = self.total_rows.partial_cmp(&other.total_rows).unwrap();
        if rows_difference.is_ne() {
            return Some(rows_difference);
        }

        // CRLF is preferred over Byte(..)
        if self.record_terminator == RecordTerminator::Crlf && other.record_terminator != RecordTerminator::Crlf {
            return Some(Ordering::Greater)
        }
        if self.record_terminator != RecordTerminator::Crlf && other.record_terminator == RecordTerminator::Crlf {
            return Some(Ordering::Less)
        }

        return Some(Ordering::Equal);
    }
}

impl Ord for SingleByteDialect {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
