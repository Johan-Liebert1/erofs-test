use std::fmt::Debug;

const S_IFMT: u16 = 0o170000;
const S_IFREG: u16 = 0o100000;
const S_IFCHR: u16 = 0o020000;
const S_IFDIR: u16 = 0o040000;
const S_IFBLK: u16 = 0o060000;
const S_IFIFO: u16 = 0o010000;
const S_IFLNK: u16 = 0o120000;
const S_IFSOCK: u16 = 0o140000;

#[repr(C)]
pub struct CompactInodeHeader {
    pub format: u16,
    pub xattr_icount: u16,
    pub mode: u16,
    pub nlink: u16,

    pub size: u32,
    pub reserved: u32,

    pub u: u32,
    pub ino: u32, // only used for 32-bit stat compatibility

    pub uid: u16,
    pub gid: u16,
    pub reserved2: [u8; 4],
}

impl Debug for CompactInodeHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "CompactInodeHeader {{")?;
        writeln!(f, "\tformat: 0b_{:08b} ({})", self.format, self.format)?;
        writeln!(f, "\txattr_icount: {}", self.xattr_icount)?;
        writeln!(f, "\tmode: {}", self.mode)?;
        writeln!(f, "\tnlink: {}", self.nlink)?;
        writeln!(f, "\tsize: {}", self.size)?;
        writeln!(f, "\treserved: {}", self.reserved)?;
        writeln!(f, "\tu: {}", self.u)?;
        writeln!(f, "\tino: {}", self.ino)?;
        writeln!(f, "\tuid: {}", self.uid)?;
        writeln!(f, "\tgid: {}", self.gid)?;
        writeln!(f, "\treserved2: {:?}", self.reserved2)?;
        writeln!(f, "}}")?;

        Ok(())
    }
}

#[repr(C)]
pub struct ExtendedInodeHeader {
    pub format: u16,
    pub xattr_icount: u16,
    pub mode: u16,
    pub reserved: u16,
    pub size: u64,

    pub u: u32,
    pub ino: u32, // only used for 32-bit stat compatibility
    pub uid: u32,
    pub gid: u32,

    pub mtime: u64,

    pub mtime_nsec: u32,
    pub nlink: u32,

    pub reserved2: [u8; 16],
}

impl Debug for ExtendedInodeHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ExtendedInodeHeader {{")?;
        writeln!(f, "\tformat: 0b_{:08b} ({})", self.format, self.format)?;
        writeln!(f, "\txattr_icount: {}", self.xattr_icount)?;
        writeln!(f, "\tmode: {}", self.mode)?;
        writeln!(f, "\treserved: {}", self.reserved)?;
        writeln!(f, "\tsize: {}", self.size)?;

        writeln!(f, "\tu: {}", self.u)?;
        writeln!(f, "\tino: {}", self.ino)?;
        writeln!(f, "\tuid: {}", self.uid)?;
        writeln!(f, "\tgid: {}", self.gid)?;

        writeln!(f, "\tmtime: {}", self.mtime)?;

        writeln!(f, "\tmtime_nsec: {}", self.mtime_nsec)?;
        writeln!(f, "\tnlink: {}", self.nlink)?;

        writeln!(f, "\treserved2: {:?}", self.reserved2)?;

        writeln!(f, "}}")?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum Inode {
    Compact(CompactInodeHeader),
    Extended(ExtendedInodeHeader),
}

impl Inode {
    pub fn mode(&self) -> u16 {
        match self {
            Inode::Compact(c) => c.mode,
            Inode::Extended(e) => e.mode,
        }
    }

    pub fn is_dir(&self) -> bool {
        (self.mode() & S_IFMT) == S_IFDIR
    }

    pub fn xattrs<'a>(&self, inode_data: &'a [u8]) -> &'a [u8] {
        match self {
            Inode::Compact(compact) if compact.xattr_icount > 0 => {
                let size = (compact.xattr_icount as usize - 1) * 4 + 12;

                // WE skip the header bit from the data and take the rest
                &inode_data[std::mem::size_of::<CompactInodeHeader>()..][..size]
            }

            Inode::Extended(extended) if extended.xattr_icount > 0 => {
                let size = (extended.xattr_icount as usize - 1) * 4 + 12;

                // WE skip the header bit from the data and take the rest
                &inode_data[std::mem::size_of::<ExtendedInodeHeader>()..][..size]
            }

            _ => &[],
        }
    }
}
