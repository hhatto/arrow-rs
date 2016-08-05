use common::status;
use buffer;
use libc;

pub enum AccessMode {
  READ_ONLY,
  READ_WRITE
}

pub enum MemorySource {}

extern "C" {
  pub fn open_mmap_src(path: *const libc::c_char, mode: AccessMode) -> *mut MemorySource;
  pub fn release_mmap_src(src: *mut MemorySource);
  pub fn close_mmap_src(src: *mut MemorySource) -> *const status::Status;
  pub fn read_at_mmap_src(src: *mut MemorySource, pos: i64, nbytes: i64) -> *const buffer::Buffer;
  pub fn write_mmap_src(src: *mut MemorySource, pos: i64, data: *const u8, nbytes: i64) -> *const status::Status;
  pub fn mmap_src_size(src: *mut MemorySource) -> i64;
}