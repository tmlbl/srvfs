use std::collections::HashMap;
use std::time::UNIX_EPOCH;

use fuser::{FileType, FileAttr};

// Data structure to keep track of virtual files created in a session
struct VFS {
    nodes: Vec<VNode>
}

impl VFS {
    pub fn new() -> VFS {
        let root = VNode {
            ino: 0,
            path: String::from("/"),
            kind: FileType::Directory,
            children: HashMap::new()
        };
        VFS {
            nodes: vec![root]
        }
    }

    pub fn lookup(&self, parent: u64, path: String) -> Option<FileAttr> {
        let parent = self.nodes.get(parent as usize);
        match parent {
            None => None,
            Some(node) => {
                match node.children.get(&path) {
                    None => None,
                    Some(ino) => {
                        Some(self.nodes.get(*ino as usize).unwrap().attr())
                    }
                }
            }
        }
    }
}

struct VNode {
    ino: u64,
    path: String,
    kind: FileType,
    children: HashMap<String, u64>,
}

impl VNode {
    pub fn attr(&self) -> FileAttr {
        FileAttr {
            ino: self.ino,
            size: 0,
            blocks: 0,
            atime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: self.kind,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_appends_and_looks_up() {
        let mut fs = VFS::new();
        let root = fs.lookup(0, String::from("/")).unwrap();
        assert!(root.ino == 0);
    }
}
