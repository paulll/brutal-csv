mod single_byte;

use std::io::{Read, Write};
pub use single_byte::{SingleByteDialectValidator, SingleByteDialect};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Dialect {
    SingleByte(SingleByteDialect)
}

pub trait DialectGroupValidator {
    fn try_process_chunk(&mut self, chunk: &[u8]) -> Result<(), String>;
    fn finalize(&mut self) -> Option<Dialect>;
}

trait Normalize {
    fn to_asv(
        &self,
        src: impl Read,
        dest: impl Write
    );
}

impl Dialect {
    pub fn to_asv(&self, src: impl Read, dest: impl Write) {
        match self {
            Dialect::SingleByte(sb) => {
                sb.to_asv(src, dest)
            }
        }
    }
}
