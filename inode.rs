use std::{
    fmt::{Debug, Display},
    usize,
};

const S_IFMT: u16 = 0o170000;
const S_IFREG: u16 = 0o100000;
const S_IFCHR: u16 = 0o020000;
const S_IFDIR: u16 = 0o040000;
const S_IFBLK: u16 = 0o060000;
const S_IFIFO: u16 = 0o010000;
const S_IFLNK: u16 = 0o120000;
const S_IFSOCK: u16 = 0o140000;

const EROFS_I_DATALAYOUT_BIT: u8 = 1;
const EROFS_I_DATALAYOUT_MASK: u8 = 0b00000111;

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct DirEnt {
    pub nid: u64,         // le
    pub name_offset: u16, // le
    pub file_type: u8,
    pub reserved: u8,
}

pub struct MyDirEnt {
    pub dirent: DirEnt,
    pub name: String,
}

impl Display for MyDirEnt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let nid = self.dirent.nid;
        let name_offset = self.dirent.name_offset;

        f.debug_struct("MyDirEnt")
            .field("name", &self.name)
            .field("nid", &nid)
            .field("name_offset", &name_offset)
            .field("file_type", &self.dirent.file_type)
            .field("reserved", &self.dirent.reserved)
            .finish()
    }
}

impl Debug for MyDirEnt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Debug)]
pub enum InodeDataLayout {
    FlatPlain,
    CompressedFull,
    FlatInline,
    CompressedCompact,
    ChunkBased,
}

impl TryFrom<u8> for InodeDataLayout {
    type Error = std::io::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(InodeDataLayout::FlatPlain),
            1 => Ok(InodeDataLayout::CompressedFull),
            2 => Ok(InodeDataLayout::FlatInline),
            3 => Ok(InodeDataLayout::CompressedCompact),
            4 => Ok(InodeDataLayout::ChunkBased),
            _ => Err(std::io::ErrorKind::Other.into()),
        }
    }
}

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

    pub fn size(&self) -> u64 {
        match self {
            Inode::Compact(c) => c.size.into(),
            Inode::Extended(e) => e.size,
        }
    }

    pub fn u(&self) -> u32 {
        match self {
            Inode::Compact(c) => c.u,
            Inode::Extended(e) => e.u,
        }
    }

    pub fn xattrs<'a>(&self, inode_data: &'a [u8]) -> &'a [u8] {
        match self {
            Inode::Compact(compact) if compact.xattr_icount > 0 => {
                let size = (compact.xattr_icount as usize - 1) * 4 + 12;
                println!("extended attr size: {size}");

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

    pub fn data_layout(&self) -> Result<InodeDataLayout, std::io::Error> {
        match self {
            Inode::Compact(c) => {
                ((c.format >> EROFS_I_DATALAYOUT_BIT) as u8 & EROFS_I_DATALAYOUT_MASK).try_into()
            }

            Inode::Extended(e) => {
                ((e.format >> EROFS_I_DATALAYOUT_BIT) as u8 & EROFS_I_DATALAYOUT_MASK).try_into()
            }
        }
    }

    /// Returns the dirent at index `num`
    fn get_dirent(&self, inode_data: &[u8], num: usize) -> DirEnt {
        let dirent_size = std::mem::size_of::<DirEnt>();

        let start = dirent_size * num;
        let end = start + dirent_size;

        if end > inode_data.len() {
            println!("dirent_num: {num:?}");
            println!("inode_data: {inode_data:?}");
            println!("inode: {self:?}");
        }

        let dirent = &inode_data[start..end];

        // NOTE: this works out with little endian as my machine is little endian
        // This would break spectacularly on a big endian machine
        let dirent = unsafe { std::ptr::read_unaligned(dirent.as_ptr() as *const DirEnt) };

        dirent
    }

    pub fn parse_dirents(&self, inode_data: &[u8]) -> Vec<MyDirEnt> {
        // Directories are stored as follows
        // [dirent0][dirent1]...[direntN][name strings...]
        //
        // where dirent
        //
        //  struct erofs_dirent {
        //      __le64 nid;     // node number
        //      __le16 nameoff; // start offset of file name
        //      __u8 file_type; // file type
        //      __u8 reserved;  // reserved
        //  } __packed;

        // To get the name of the dirent, we have to parse the next dirent, find the name offset,
        // then subtract the current name offset to get the name length

        let mut dirents = vec![];
        let mut dirent_num = 0;

        let mut first = self.get_dirent(inode_data, dirent_num);
        let first_nameoff = first.name_offset;

        // There will always be two dirents, "." and ".."
        dirent_num += 1;
        let mut second = self.get_dirent(inode_data, dirent_num);

        assert!(first_nameoff % 12 == 0);
        let num_dirents = (first_nameoff / 12) as usize;

        println!("num_dirents: {num_dirents:?}");

        if num_dirents == 0 {
            println!("first: {first:?}");
            return vec![];
        }

        loop {
            let my_dirent = MyDirEnt {
                dirent: first.clone(),
                name: String::from_utf8(
                    inode_data[first.name_offset as usize..second.name_offset as usize].into(),
                )
                .unwrap(),
            };

            dirents.push(my_dirent);

            first = second;

            dirent_num += 1;

            if dirent_num >= num_dirents {
                break;
            }

            second = self.get_dirent(inode_data, dirent_num);
        }

        // We will still be missing the last dirent here
        let my_dirent = MyDirEnt {
            dirent: first.clone(),
            name: String::from_utf8(
                inode_data[first.name_offset as usize..]
                    .split(|x| *x == 0)
                    .next()
                    .unwrap()
                    .into(),
            )
            .unwrap(),
        };

        dirents.push(my_dirent);

        return dirents;
    }

    pub fn dirents(&self, layout: InodeDataLayout, inode_data: &[u8]) -> Vec<MyDirEnt> {
        let header_size = match self {
            Inode::Compact(..) => std::mem::size_of::<CompactInodeHeader>(),
            Inode::Extended(..) => std::mem::size_of::<ExtendedInodeHeader>(),
        };

        use InodeDataLayout::*;

        let data = match layout {
            FlatPlain => {
                println!("FlatPlain inode: {self:#?}");

                let data = &inode_data[self.u() as usize..][..self.size() as usize];

                self.parse_dirents(&data)
            }

            CompressedFull => unimplemented!("CompressedFull"),

            // Data is stored right after the xattrs
            FlatInline => {
                // TODO: Handle xattrs
                let data = &inode_data[header_size..][..self.size() as usize];
                self.parse_dirents(&data)
            }

            CompressedCompact => unimplemented!("CompressedCompact"),
            ChunkBased => unimplemented!("ChunkBased"),
        };

        data
    }
}
