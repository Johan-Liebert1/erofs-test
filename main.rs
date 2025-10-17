#![allow(unused_assignments)]
#![allow(dead_code)]

mod inode;
mod sb;
mod utils;

use inode::*;
use sb::*;
use std::ptr;

use anyhow::Result;

const COMPOSEFS_MAGIC: u32 = 0xd078629a;
const COMPOSEFS_VERSION: u32 = 2;

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

fn get_inode_offset(superblock: &Superblock, nid: usize) -> usize {
    let block_size = 1 << superblock.blkszbits;

    // inode offset = meta_blkaddr * block_size + 32 * nid
    superblock.meta_blkaddr as usize * block_size as usize + 32usize * nid
}

fn main() -> Result<()> {
    let file = std::fs::read("./file.erofs")?;
    let file: &[u8] = &file;

    println!("file_len: {}", file.len());

    // First 1kb is our data
    // We can put anything in here
    let header = &file[..1024];
    assert_header(header);

    // After header of 1KB we have the superblock
    let superblock = &file[1024..2048];
    assert_superblock(superblock)?;

    let sb_bytes: &[u8; 1024] = superblock.try_into().expect("slice must be 1024 bytes");
    let superblock: Superblock =
        unsafe { ptr::read_unaligned(sb_bytes.as_ptr() as *const Superblock) };

    let block_size = 1 << superblock.blkszbits;

    println!("superblock: {superblock:#?}");
    println!("block_size: {}", block_size);

    let mut nids = vec![superblock.root_nid as usize];

    while nids.len() > 0 {
        let nid = nids[0];
        nids = nids[1..].into();

        let inode_start = get_inode_offset(&superblock, nid);

        let inode_first_byte = file[inode_start];

        let bytes = &file[inode_start..];

        let inode = if inode_first_byte & 1 == 1 {
            // extended inode
            let bytes = &bytes[..std::mem::size_of::<ExtendedInodeHeader>()];
            let ext = unsafe { ptr::read_unaligned(bytes.as_ptr() as *const ExtendedInodeHeader) };

            Inode::Extended(ext)
        } else {
            let bytes = &bytes[..std::mem::size_of::<CompactInodeHeader>()];
            let cpt = unsafe { ptr::read_unaligned(bytes.as_ptr() as *const CompactInodeHeader) };

            Inode::Compact(cpt)
        };

        if !inode.is_dir() {
            continue;
        }

        let dirents = inode.dirents(inode.data_layout()?, &file[inode_start..]);

        println!("dirents: {dirents:#?}");

        for dirent in dirents {
            if dirent.name == "." || dirent.name == ".." {
                continue;
            }

            nids.push(dirent.dirent.nid as usize);
        }
    }

    Ok(())
}
