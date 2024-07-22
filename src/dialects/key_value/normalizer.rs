use std::io::{Read, Write};
use crate::dialects::{Normalize};
use crate::dialects::key_value::KeyValueDialect;

impl Normalize for KeyValueDialect {
    fn to_asv(&self, src: impl Read, dst: impl Write) {
        let mut normalizer = KeyValueDialectNormalizer::new(
            src,
            dst,
            self.clone()
        );

        normalizer.normalize();
    }
}
struct KeyValueDialectNormalizer <W: Write, R: Read> {
    writer: W,
    reader: R,
    dialect: KeyValueDialect,
    current_column: usize,
}

impl<W: Write, R: Read> KeyValueDialectNormalizer<W, R> {
    fn new(reader: R, writer: W, dialect: KeyValueDialect) -> Self {
        Self {
            writer,
            reader,
            dialect,
            current_column: 0,
        }
    }

    fn normalize(&mut self) {
        self.writer.write_all(b"login\x1fpassword\x1e").unwrap();

        let mut buffer = vec![b'0'; 1024*1024*16]; // 16 MiB chunks
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
            if self.try_next_row(c) {
                continue;
            }

            if self.try_next_field(c) {
                continue;
            }

            self.try_next_char(c);
        }
    }

    #[inline]
    fn try_next_row(&mut self, c: &u8) -> bool {
        if *c == b'\r' {
            return true; // consume CR byte, but no line break here yet
        } else if *c == b'\n'{
            self.end_row();
            true
        } else {
            false
        }
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
        self.writer.write_all(&[*c]).unwrap();
        true
    }

    #[inline]
    fn end_field(&mut self) {
        if self.current_column == 0 {
            self.writer.write_all(b"\x1f").unwrap();
            self.current_column = 1;
        }
    }


    #[inline]
    fn end_row(&mut self) {
        self.current_column = 0;
        self.writer.write_all(b"\x1e").unwrap();
    }
}

