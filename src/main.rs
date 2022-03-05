use fuser::{Filesystem, FileAttr, Request, 
    ReplyEntry, ReplyDirectory, ReplyAttr, FileType, MountOption};
use std::ffi::OsStr;
use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};
use std::fs;

mod vfs;

const TTL: Duration = Duration::from_secs(1);

struct SrvFS {
    v: vfs::VFS,
}

impl SrvFS {
    fn new() -> SrvFS {
        SrvFS {
            v: vfs::VFS::new()
        }
    }
}

impl Filesystem for SrvFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        println!("LOOKUP {}", name.to_str().unwrap());
        match self.v.lookup(parent, name.to_str().unwrap()) {
            None => reply.error(libc::ENOENT),
            Some(attr) => reply.entry(&TTL, &attr, 1),
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        println!("ATTR for node {}", ino);
        match self.v.nodes.get(ino as usize) {
            Some(node) => reply.attr(&TTL, &node.attr()),
            None => reply.error(libc::ENOENT),
        }
    }

    fn readdir(&mut self, _req: &Request, ino: u64, fh: u64,
               offset: i64, mut reply: ReplyDirectory) {
 
        println!("Reading dir with ino {} offset {}", ino, offset);
        if offset > 0 {
            reply.ok();
            return
        }
        let mut full: bool;
        for node in &self.v.children(ino) {
            full = reply.add(node.ino, 1, node.kind, node.path.clone());
            if full {
                break
            }
        }
        reply.ok()
    }

    fn mkdir(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        _mode: u32,
        _umask: u32,
        reply: ReplyEntry,
    ) {
        self.v.create(parent, name.to_str().unwrap(), FileType::Directory);
        let attr = self.v.lookup(parent, name.to_str().unwrap()).unwrap();
        reply.entry(&TTL, &attr, 1);
    }

}

fn main() {
    let mountpoint = "/tmp/srvfs";
    fs::create_dir_all(mountpoint).unwrap();

    let options = vec![MountOption::FSName(String::from("srvfs")),
        MountOption::RW, MountOption::AutoUnmount];

    fuser::mount2(SrvFS::new(), mountpoint, &options).unwrap();
}
