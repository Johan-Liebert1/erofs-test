#![allow(unused_assignments)]

use anyhow::Result;

const COMPOSEFS_MAGIC: u32 = 0xd078629a;
const COMPOSEFS_VERSION: u32 = 2;

const MAGIC_V1: u32 = 0xE0F5E1E2;
const BLOCK_BITS: u8 = 12;

const FEATURE_COMPAT_MTIME: u32 = 2;
const FEATURE_COMPAT_XATTR_FILTER: u32 = 4;

#[fn_error_context::context("For u64 {msg}")]
fn u64_le(buf: &[u8], msg: &str) -> Result<u64> {
    let thing: &[u8; 8] = buf[..8].try_into().unwrap();
    Ok(u64::from_le_bytes(*thing))
}

#[fn_error_context::context("For u32 {msg}")]
fn u32_le(buf: &[u8], msg: &str) -> Result<u32> {
    let thing: &[u8; 4] = buf[..4].try_into().unwrap();
    Ok(u32::from_le_bytes(*thing))
}

#[fn_error_context::context("For u16 {msg}")]
fn u16_le(buf: &[u8], msg: &str) -> Result<u16> {
    let thing: &[u8; 2] = buf[..2].try_into().unwrap();
    Ok(u16::from_le_bytes(*thing))
}

fn assert_header(mut header: &[u8]) {
    // Composefs header is u32
    let compsefs_header: &[u8; 4] = &header[..4].try_into().unwrap();
    let magic_num = u32::from_le_bytes(*compsefs_header);
    assert_eq!(magic_num, COMPOSEFS_MAGIC);
    header = &header[4..];

    // Then we have a u32 version
    let version: &[u8; 4] = &header[..4].try_into().unwrap();
    let version = u32::from_le_bytes(*version);
    assert_eq!(version, 1);
    header = &header[4..];

    // Then we have all zero flags, u32 again
    let flags: &[u8; 4] = &header[..4].try_into().unwrap();
    let flags = u32::from_le_bytes(*flags);
    assert_eq!(flags, 0);
    header = &header[4..];

    // Then we have composefs_version, u32 again
    let composefs_version: &[u8; 4] = &header[..4].try_into().unwrap();
    let composefs_version = u32::from_le_bytes(*composefs_version);
    assert_eq!(composefs_version, COMPOSEFS_VERSION);
}

#[derive(Debug)]
#[repr(C)]
pub struct Superblock {
    pub magic: u32,
    pub checksum: u32,
    pub feature_compat: u32,
    pub blkszbits: u8,
    pub extslots: u8,
    pub root_nid: u16,

    pub inos: u64,
    pub build_time: u64,

    pub build_time_nsec: u32,
    pub blocks: u32,
    pub meta_blkaddr: u32,
    pub xattr_blkaddr: u32,

    pub uuid: [u8; 16],

    pub volume_name: [u8; 16],

    pub feature_incompat: u32,
    pub available_compr_algs: u16,
    pub extra_devices: u16,
    pub devt_slotoff: u16,
    pub dirblkbits: u8,
    pub xattr_prefix_count: u8,
    pub xattr_prefix_start: u32,

    pub packed_nid: u64,
    pub xattr_filter_reserved: u8,
    pub reserved2: [u8; 23],
}

fn assert_superblock(mut superblock: &[u8]) -> Result<()> {
    // First is superblock magic
    assert_eq!(u32_le(superblock, "superblock_magic")?, MAGIC_V1);
    superblock = &superblock[4..];

    assert_eq!(u32_le(superblock, "sb_checksum")?, 0);
    superblock = &superblock[4..];

    assert_eq!(
        u32_le(superblock, "sb_feature_compat")?,
        FEATURE_COMPAT_MTIME | FEATURE_COMPAT_XATTR_FILTER
    );
    superblock = &superblock[4..];

    // blkszbits
    assert_eq!(superblock[0], BLOCK_BITS);
    superblock = &superblock[1..];

    // extslots
    assert_eq!(superblock[0], 0);
    superblock = &superblock[1..];

    // TODO: Idk where this 36 comes from
    assert_eq!(u16_le(superblock, "sb_root_nid")?, 36);
    superblock = &superblock[2..];

    let inos = u64_le(superblock, "sb_inos")?;
    println!("num inodes: {inos}");
    superblock = &superblock[8..];

    let build_time = u64_le(superblock, "sb_build_time")?;
    println!("build time: {build_time}");
    superblock = &superblock[8..];

    Ok(())
}

fn main() -> Result<()> {
    let file = std::fs::read("./file.erofs")?;
    let mut file: &[u8] = &file;

    println!("file_len: {}", file.len());

    // First 1kb is our data
    // We can put anything in here
    let header = &file[..1024];
    assert_header(header);

    println!("sizeof superblock: {}", std::mem::size_of::<Superblock>());

    // file = &file[1024..];

    // After header of 1KB we have the superblock
    let superblock = &file[1024..2048];
    assert_superblock(superblock)?;

    // file = &file[1024..];

    let sb_bytes: &[u8; 1024] = superblock.try_into().expect("slice must be 1024 bytes");
    let superblock: Superblock = unsafe {
        std::ptr::read_unaligned(sb_bytes.as_ptr() as *const Superblock)
    };

    println!("superblock: {superblock:#?}");
    println!("block_size: {}", 1 << superblock.blkszbits);

    println!("{:?}", &file[..4096]);

    Ok(())
}
