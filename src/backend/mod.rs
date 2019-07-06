use std::io::{self, Read};

use bytehash::ByteHash;

mod disk;
mod mem;

pub use self::disk::DiskBackend;
pub use self::mem::MemBackend;

pub enum PutResult {
    Ok,
    AlreadyThere,
}

pub trait Backend<H: ByteHash>: Send + Sync {
    fn get<'a>(&'a self, digest: &H::Digest) -> io::Result<Box<dyn Read + 'a>>;

    /// Put the serialized value in the backend.
    fn put(&self, digest: H::Digest, bytes: Vec<u8>) -> io::Result<PutResult>;

    // fn clear(&self) -> io::Result<()>;

    fn size(&self) -> usize;
}
