use fuser::{Filesystem, Request, 
    ReplyEntry, ReplyDirectory, ReplyAttr, 
    ReplyCreate, ReplyWrite, ReplyData, FileType, MountOption};
use std::ffi::OsStr;
use std::time::Duration;
use std::fs;
use std::collections::HashMap;

use crate::vfs::VFS;

const HELLO_TXT_CONTENT: &str = "Hello 0hg843hg08uh World!\n";

const TTL: Duration = Duration::from_secs(1);

pub struct SrvFS {
    v: VFS,
    nc: nats::Connection,
    subs: HashMap<String, nats::Subscription>
}

impl SrvFS {
    fn new(nc: nats::Connection) -> SrvFS {
        SrvFS {
            v: VFS::new(),
            nc,
            subs: HashMap::new(),
        }
    }

    fn get_sub(&mut self, subject: &str) -> Option<&nats::Subscription> {
        if !self.subs.contains_key(subject) {
            let sub = self.nc.subscribe(subject).unwrap();
            self.subs.insert(subject.to_string(), sub);
        }
        return self.subs.get(subject);
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

    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64,
               offset: i64, mut reply: ReplyDirectory) {
 
        println!("Reading dir with ino {} offset {}", ino, offset);
        if offset > 0 {
            reply.ok();
            return
        }
        let mut full: bool;
        for node in &self.v.children(ino) {
            full = reply.add(node.ino, 1, node.kind, node.name.clone());
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

    fn create(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        _mode: u32,
        _umask: u32,
        _flags: i32,
        reply: ReplyCreate,
    ) {
        self.v.create(parent, name.to_str().unwrap(), FileType::RegularFile);
        let attr = self.v.lookup(parent, name.to_str().unwrap()).unwrap();
        reply.created(&TTL, &attr, 1, attr.ino, libc::O_CREAT as u32);
    }

    /// Set file attributes.
    fn setattr(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _mode: Option<u32>,
        _uid: Option<u32>,
        _gid: Option<u32>,
        _size: Option<u64>,
        _atime: Option<fuser::TimeOrNow>,
        _mtime: Option<fuser::TimeOrNow>,
        _ctime: Option<std::time::SystemTime>,
        _fh: Option<u64>,
        _crtime: Option<std::time::SystemTime>,
        _chgtime: Option<std::time::SystemTime>,
        _bkuptime: Option<std::time::SystemTime>,
        _flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        match self.v.nodes.get(ino as usize) {
            Some(node) => reply.attr(&TTL, &node.attr()),
            None => reply.error(libc::ENOENT),
        }
    }

    // Write data
    fn write(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        _offset: i64,
        data: &[u8],
        _write_flags: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyWrite,
    ) {
        match self.v.nodes.get(ino as usize) {
            Some(node) => {
                println!("Publishing message with subject: {}", node.path);
                self.nc.publish(node.path.as_str(), data).unwrap();
                reply.written(data.len() as u32);
            }
            None => reply.error(libc::ENOENT),
        }
    }

    fn read(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        let node = self.v.nodes.get(ino as usize).unwrap();
        let path = node.path.clone();
        let sub = self.get_sub(path.as_str()).unwrap();
        let msg = sub.messages().next().unwrap();

        reply.data(&msg.data);
    }
}

pub fn mount(mountpoint: &str, nc: nats::Connection) {
    fs::create_dir_all(mountpoint).unwrap();

    let options = vec![MountOption::FSName(String::from("srvfs")),
        MountOption::RW, MountOption::AutoUnmount];

    fuser::mount2(SrvFS::new(nc), mountpoint, &options).unwrap();
}
