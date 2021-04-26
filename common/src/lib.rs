mod errors;

pub use errors::Error;

use serde::{Deserialize, Serialize};

pub const MEMORY_START: usize = 1;
pub const LEN_SIZE: usize = std::mem::size_of::<u32>();

#[derive(Debug, Deserialize, Serialize)]
pub struct Plugin {
    pub address: u32,
    pub name: String,
}

pub fn write<T>(data: &T) -> Result<u32, Error> where T: Serialize {
    let dst = MEMORY_START as *mut u8;
    let data = bincode::serialize(&data)?;
    let len_bytes = (data.len() as u32).to_ne_bytes();
    unsafe {
        std::ptr::copy(len_bytes.as_ptr(), dst, LEN_SIZE);
        std::ptr::copy(data.as_ptr(), dst.offset(LEN_SIZE as _), data.len());
    }
    Ok(dst as _)
}
