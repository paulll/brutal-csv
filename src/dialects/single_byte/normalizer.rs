use std::io::{Read, Write};
use crate::dialects::{Normalize, SingleByteDialect};
use crate::dialects::single_byte::RecordTerminator;

impl Normalize for SingleByteDialect {
    fn to_asv(&self, src: impl Read, dst: impl Write) {
        let mut normalizer = SingleByteDialectNormalizer::new(
            src,
            dst,
            self.clone()
        );

        normalizer.normalize();
    }
}
struct SingleByteDialectNormalizer <W: Write, R: Read> {
    writer: W,
    reader: R,
    dialect: SingleByteDialect,
    escape_active: bool,
    quote_active: bool,
    current_column: usize,
    prev_char_was_cr: bool,
    is_first_row: bool,
}

impl<W: Write, R: Read> SingleByteDialectNormalizer<W, R> {
    fn new(reader: R, writer: W, dialect: SingleByteDialect) -> Self {
        Self {
            writer,
            reader,
            dialect,
            escape_active: false,
            quote_active: false,
            current_column: 0,
            prev_char_was_cr: false,
            is_first_row: true,
        }
    }

    fn normalize(&mut self) {
        let mut buffer = vec![b'0'; 1024*1024*16]; // 16 MiB chunks
        const PLACEHOLDER: &[u8] = b"__NO_HEADER__\x1f";

        if self.dialect.header.is_none() {
            let mut header: Vec<u8> = self.dialect.empty_columns
                .iter()
                .flat_map(|_| PLACEHOLDER)
                .copied()
                .collect();

            *header.last_mut().unwrap() = b'\x1e';
            self.writer.write_all(&header).unwrap();
        }

        loop {
            let chunk_size = self.reader.read(&mut buffer).unwrap();
            if chunk_size == 0 {
                break
            }

            self.process_chunk(&buffer[0..chunk_size]);
        }
    }

    fn process_chunk(&mut self, chunk: &[u8]) {
        for c in chunk {
            // these try_* functions returns true if byte is accepted/consumed

            if self.dialect.has_escaped_line_breaks && self.try_escape(c) {
                continue
            }

            if self.dialect.has_quoted_line_breaks && self.try_quote(c) {
                continue
            }

            if self.try_next_row(c) {
                continue;
            }

            if !self.dialect.has_escaped_line_breaks && self.try_escape(c) {
                continue;
            }

            if !self.dialect.has_quoted_line_breaks && self.try_quote(c) {
                continue;
            }

            if self.try_next_field(c) {
                continue;
            }

            self.try_next_char(c);
        }
    }

    #[inline]
    fn try_escape(&mut self, c: &u8) -> bool {
        if self.escape_active {
            self.escape_active = false;
            self.writer.write_all(&[*c]).unwrap();
            return true
        }

        if let Some(e) = self.dialect.escape_char {
            if *c == e {
                self.escape_active = true;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    #[inline]
    fn try_quote(&mut self, c: &u8) -> bool {
        if let Some(q) = self.dialect.quote_char {
            let was_active = self.quote_active;
            let should_switch = q == *c;

            if should_switch {
                self.quote_active = !self.quote_active;
                return true
            } else if was_active {
                self.writer.write_all(&[*c]).unwrap();
                return true
            }
        }
        false
    }

    #[inline]
    fn try_next_row(&mut self, c: &u8) -> bool {
        let is_break = match &self.dialect.record_terminator {
            RecordTerminator::Byte(t) => {
                c == t
            }
            RecordTerminator::Crlf => {
                if *c == b'\r' {
                    self.prev_char_was_cr = true;
                    return true; // consume CR byte, but no line break here yet
                } else if *c == b'\n' && self.prev_char_was_cr {
                    self.prev_char_was_cr = false;
                    true
                } else {
                    self.prev_char_was_cr = false;
                    false
                }
            }
        };

        if is_break {
            self.end_row();
        }

        is_break
    }

    #[inline]
    fn try_next_field(&mut self, c: &u8) -> bool {
        if *c == self.dialect.field_separator {
            self.end_field();
            true
        } else {
            false
        }
    }

    #[inline]
    fn try_next_char(&mut self, c: &u8) -> bool {
        if self.is_first_row && !self.should_emit_current_column() {
            return true
        }

        self.writer.write_all(&[*c]).unwrap();
        true
    }

    #[inline]
    fn end_field(&mut self) {
        self.quote_active = false;
        self.escape_active = false;

        // skip empty columns
        let should_emit= self.should_emit_current_column();
        self.current_column += 1;

        // no emit delimiter after last column
        if self.current_column == self.dialect.empty_columns.len() {
            return
        }

        if should_emit {
            self.writer.write_all(b"\x1f").unwrap();
        }
    }

    #[inline]
    fn should_emit_current_column(&self) -> bool {
        if let Some(c) = self.dialect.empty_columns.get(self.current_column) {
            !c
        } else {
            false
        }
    }

    #[inline]
    fn end_row(&mut self) {
        self.end_field();
        debug_assert_eq!(self.current_column, self.dialect.empty_columns.len());


        self.prev_char_was_cr = false;
        self.current_column = 0;
        self.is_first_row = false;
        self.writer.write_all(b"\x1e").unwrap();
    }
}

