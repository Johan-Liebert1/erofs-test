use crate::utils::*;
use anyhow::Result;
use std::fmt::Debug;

const MAGIC_V1: u32 = 0xE0F5E1E2;
const BLOCK_BITS: u8 = 12;

const FEATURE_COMPAT_MTIME: u32 = 2;
const FEATURE_COMPAT_XATTR_FILTER: u32 = 4;

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

impl Debug for Superblock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Superblock {{ ")?;
        writeln!(f, "\tmagic: {}", self.magic)?;
        writeln!(f, "\tchecksum: {}", self.checksum)?;
        writeln!(f, "\tfeature_compat: {}", self.feature_compat)?;
        writeln!(f, "\tblkszbits: {}", self.blkszbits)?;
        writeln!(f, "\textslots: {}", self.extslots)?;
        writeln!(f, "\troot_nid: {}", self.root_nid)?;
        writeln!(f, "\tinos: {}", self.inos)?;
        writeln!(f, "\tbuild_time: {}", self.build_time)?;
        writeln!(f, "\tbuild_time_nsec: {}", self.build_time_nsec)?;
        writeln!(f, "\tblocks: {}", self.blocks)?;
        writeln!(f, "\tmeta_blkaddr: {}", self.meta_blkaddr)?;
        writeln!(f, "\txattr_blkaddr: {}", self.xattr_blkaddr)?;
        writeln!(f, "\tuuid: {:?}", self.uuid)?;
        writeln!(f, "\tvolume_name: {:?}", self.volume_name)?;
        writeln!(f, "\tfeature_incompat: {}", self.feature_incompat)?;
        writeln!(f, "\tavailable_compr_algs: {}", self.available_compr_algs)?;
        writeln!(f, "\textra_devices: {}", self.extra_devices)?;
        writeln!(f, "\tdevt_slotoff: {}", self.devt_slotoff)?;
        writeln!(f, "\tdirblkbits: {}", self.dirblkbits)?;
        writeln!(f, "\txattr_prefix_count: {}", self.xattr_prefix_count)?;
        writeln!(f, "\txattr_prefix_start: {}", self.xattr_prefix_start)?;
        writeln!(f, "\tpacked_nid: {}", self.packed_nid)?;
        writeln!(f, "\txattr_filter_reserved: {}", self.xattr_filter_reserved)?;
        writeln!(f, "\treserved2: {:?}", self.reserved2)?;
        writeln!(f, "}}")
    }
}

pub fn assert_superblock(mut superblock: &[u8]) -> Result<()> {
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
