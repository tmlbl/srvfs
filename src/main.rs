use fuser::{Filesystem, FileAttr, Request, 
    ReplyEntry, ReplyDirectory, ReplyAttr, FileType, MountOption};
use std::ffi::OsStr;
use std::time::{Duration, UNIX_EPOCH};
use std::fs;

const TTL: Duration = Duration::from_secs(1);

trait VFile {
    fn attr(&self) -> FileAttr;
}

struct VPubFile {
    ino: u64
}

impl VFile for VPubFile {
    fn attr(&self) -> FileAttr {
        FileAttr {
            ino: self.ino,
            size: 0,
            blocks: 0,
            atime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: FileType::RegularFile,
            perm: 0o755,
            nlink: 2,
            uid: 501,
            gid: 20,
            rdev: 0,
            flags: 0,
            blksize: 512,
        }
    }
}

struct SrvFS {

}

impl SrvFS {
    fn attr_for_ino(&mut self, ino: u64) -> Option<FileAttr> {
        let mut attr = FileAttr {
            ino,
            size: 0,
            blocks: 0,
            atime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: FileType::Directory,
            perm: 0o755,
            nlink: 2,
            uid: 501,
            gid: 20,
            rdev: 0,
            flags: 0,
            blksize: 512,
        };

        if ino == 1 {
            return Some(attr)
        }

        return None;
    }
}

impl Filesystem for SrvFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
       reply.error(libc::ENOENT); 
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        match self.attr_for_ino(ino) {
            Some(attr) => reply.attr(&TTL, &attr),
            None => reply.error(libc::ENOENT),
        }
    }

    fn readdir(&mut self, _req: &Request, ino: u64, fh: u64,
               offset: i64, reply: ReplyDirectory) {
 
        println!("Reading dir with ino {}", ino);        
        reply.ok();
    }
}

fn main() {
    let mountpoint = "/tmp/srvfs";
    fs::create_dir_all(mountpoint).unwrap();

    let options = vec![MountOption::FSName(String::from("srvfs")),
        MountOption::RW, MountOption::AutoUnmount];

    fuser::mount2(SrvFS{}, mountpoint, &options).unwrap();
}
