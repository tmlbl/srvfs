use std::collections::HashMap;
use std::time::UNIX_EPOCH;

use fuser::{FileType, FileAttr};

// Data structure to keep track of virtual files created in a session
pub struct VFS {
    pub nodes: Vec<VNode>
}

impl VFS {
    pub fn new() -> VFS {
        let empty = VNode {
            ino: 0,
            path: String::from(""),
            kind: FileType::Symlink,
            children: HashMap::new()
        };
        let root = VNode {
            ino: 1,
            path: String::from("/"),
            kind: FileType::Directory,
            children: HashMap::new()
        };
        VFS {
            nodes: vec![empty, root]
        }
    }

    pub fn create(&mut self, parent: u64, path: &str, kind: FileType) {
        let ino = self.nodes.len() as u64;
        let node = VNode {
            ino,
            path: String::from(path),
            kind,
            children: HashMap::new()
        };
        self.nodes.push(node);
        let parent = self.nodes.get_mut(parent as usize).unwrap();
        parent.children.insert(path.to_string(), ino);
    }

    pub fn lookup(&self, parent: u64, path: &str) -> Option<FileAttr> {
        let parent = self.nodes.get(parent as usize);
        match parent {
            None => None,
            Some(node) => {
                match node.children.get(path) {
                    None => None,
                    Some(ino) => {
                        Some(self.nodes.get(*ino as usize).unwrap().attr())
                    }
                }
            }
        }
    }

    pub fn children(&self, ino: u64) -> Vec<VNode> {
        let node = self.nodes.get(ino as usize).unwrap();
        let mut v = Vec::new();
        for (name, i) in &node.children {
            let n = self.nodes.get(*i as usize).unwrap();
            v.push(VNode {
                ino: n.ino,
                path: name.clone(),
                kind: n.kind,
                children: HashMap::new(),
            })
        }
        v
    }
}

pub struct VNode {
    pub ino: u64,
    pub path: String,
    pub kind: FileType,
    children: HashMap<String, u64>,
}

impl VNode {
    pub fn attr(&self) -> FileAttr {
        let perm = match self.kind {
            FileType::Directory => 0o755,
            FileType::RegularFile => 0o644,
            _ => 0o444,
        };
        FileAttr {
            ino: self.ino,
            size: 0,
            blocks: 0,
            atime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: self.kind,
            perm,
            nlink: 2,
            uid: 501,
            gid: 20,
            rdev: 0,
            flags: 0,
            blksize: 512,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_appends_and_looks_up() {
        let mut fs = VFS::new();
        fs.create(0, "foo", FileType::RegularFile);
        let foo = fs.lookup(0, "foo");
        assert!(foo.is_some());
        let foo = foo.unwrap();
        assert_eq!(foo.ino, 2);
        assert_eq!(foo.perm, 0o644);
    }
}
