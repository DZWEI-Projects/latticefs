//! Read-only FUSE filesystem for LatticeFS.

use crate::error::Result;
use crate::fuse::inode::{
    inode_for_view_name, InodeMapper, PROJECTS_INODE, RECENT_INODE, ROOT_INODE, VIEWS_INODE,
};
use crate::model::ObjectID;
use crate::repo::LatticeRepo;
use crate::views::{BuiltinView, BuiltinViews, DynamicView};
use crate::{has_executable_tag, trust_level};
use fuser::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, ReplyOpen,
    Request,
};
use lru::LruCache;
use std::ffi::OsStr;
use std::num::NonZeroUsize;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

const TTL: Duration = Duration::from_secs(1);

#[derive(Debug, Clone)]
enum ViewKind {
    Builtin(BuiltinView),
    Dynamic(String),
}

pub struct LatticeFS {
    repo: LatticeRepo,
    inode_cache: Mutex<LruCache<u64, ObjectID>>,
    object_cache: Mutex<LruCache<ObjectID, u64>>,
}

impl LatticeFS {
    pub fn new(repo: LatticeRepo) -> Self {
        let capacity = NonZeroUsize::new(10_000).expect("non-zero");
        Self {
            repo,
            inode_cache: Mutex::new(LruCache::new(capacity)),
            object_cache: Mutex::new(LruCache::new(capacity)),
        }
    }

    fn inode_mapper(&self) -> InodeMapper<'_> {
        InodeMapper::new(&self.repo.metadata)
    }

    fn inode_for_object(&self, object_id: &ObjectID) -> Result<u64> {
        if let Some(inode) = self.object_cache.lock().unwrap().get(object_id).copied() {
            return Ok(inode);
        }

        let mapper = self.inode_mapper();
        let inode = mapper.inode_for_object(object_id)?;
        self.object_cache.lock().unwrap().put(*object_id, inode);
        self.inode_cache.lock().unwrap().put(inode, *object_id);
        Ok(inode)
    }

    fn object_id_for_inode(&self, inode: u64) -> Result<Option<ObjectID>> {
        if let Some(object_id) = self.inode_cache.lock().unwrap().get(&inode).copied() {
            return Ok(Some(object_id));
        }

        let mapper = self.inode_mapper();
        let object_id = mapper.object_id_for_inode(inode)?;
        if let Some(object_id) = object_id {
            self.object_cache.lock().unwrap().put(object_id, inode);
            self.inode_cache.lock().unwrap().put(inode, object_id);
            return Ok(Some(object_id));
        }
        Ok(None)
    }

    fn dir_attr(&self, ino: u64) -> FileAttr {
        let now = SystemTime::now();
        FileAttr {
            ino,
            size: 0,
            blocks: 0,
            atime: now,
            mtime: now,
            ctime: now,
            crtime: now,
            kind: FileType::Directory,
            perm: 0o555,
            nlink: 2,
            uid: unsafe { libc::geteuid() },
            gid: unsafe { libc::getegid() },
            rdev: 0,
            flags: 0,
            blksize: 512,
        }
    }

    fn file_attr(&self, object_id: &ObjectID, ino: u64) -> Result<FileAttr> {
        let object = self.repo.metadata.load_object(object_id)?;
        let version = self.repo.metadata.load_version(&object.current_version)?;
        let size = version.size_bytes;
        let timestamp = micros_to_systemtime(version.created_at);

        let exec = has_executable_tag(&object.tags);
        let trust = trust_level(&object.tags);
        let perm = if exec && trust >= 90 { 0o555 } else { 0o444 };

        Ok(FileAttr {
            ino,
            size,
            blocks: (size + 511) / 512,
            atime: timestamp,
            mtime: timestamp,
            ctime: timestamp,
            crtime: timestamp,
            kind: FileType::RegularFile,
            perm,
            nlink: 1,
            uid: unsafe { libc::geteuid() },
            gid: unsafe { libc::getegid() },
            rdev: 0,
            flags: 0,
            blksize: 512,
        })
    }

    fn view_for_inode(&self, inode: u64) -> Result<Option<ViewKind>> {
        if inode == PROJECTS_INODE {
            return Ok(Some(ViewKind::Builtin(BuiltinView::Projects)));
        }
        if inode == RECENT_INODE {
            return Ok(Some(ViewKind::Builtin(BuiltinView::Recent)));
        }

        // Built-in views under /views
        for view in BuiltinView::all() {
            let inode_for = inode_for_view_name(&view.name().to_lowercase());
            if inode == inode_for {
                return Ok(Some(ViewKind::Builtin(*view)));
            }
        }

        // Dynamic views
        for view in self.repo.metadata.list_views()? {
            let inode_for = inode_for_view_name(&view.name);
            if inode == inode_for {
                return Ok(Some(ViewKind::Dynamic(view.name)));
            }
        }

        Ok(None)
    }

    fn list_root_entries(&self) -> Vec<(u64, FileType, String)> {
        vec![
            (VIEWS_INODE, FileType::Directory, "views".to_string()),
            (PROJECTS_INODE, FileType::Directory, "projects".to_string()),
            (RECENT_INODE, FileType::Directory, "recent".to_string()),
        ]
    }

    fn list_views_entries(&self) -> Result<Vec<(u64, FileType, String)>> {
        let mut entries = Vec::new();

        for view in BuiltinView::all() {
            let name = view.name().to_lowercase();
            entries.push((inode_for_view_name(&name), FileType::Directory, name));
        }

        for view in self.repo.metadata.list_views()? {
            entries.push((
                inode_for_view_name(&view.name),
                FileType::Directory,
                view.name,
            ));
        }

        Ok(entries)
    }

    fn resolve_view_objects(&self, view: &ViewKind) -> Result<Vec<ObjectID>> {
        match view {
            ViewKind::Builtin(builtin) => BuiltinViews::new(&self.repo.metadata).evaluate(*builtin),
            ViewKind::Dynamic(name) => {
                let view = self.repo.metadata.load_view(name)?;
                let mut dynamic = DynamicView::new(&view.query, &self.repo.metadata)?;
                dynamic.evaluate()
            }
        }
    }

    fn object_in_view(&self, view: &ViewKind, object_id: &ObjectID) -> Result<bool> {
        let objects = self.resolve_view_objects(view)?;
        Ok(objects.iter().any(|id| id == object_id))
    }
}

impl Filesystem for LatticeFS {
    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let name_str = match name.to_str() {
            Some(s) => s,
            None => {
                reply.error(libc::ENOENT);
                return;
            }
        };

        if parent == ROOT_INODE {
            for (ino, kind, entry_name) in self.list_root_entries() {
                if entry_name == name_str {
                    let attr = match kind {
                        FileType::Directory => self.dir_attr(ino),
                        _ => self.dir_attr(ino),
                    };
                    reply.entry(&TTL, &attr, 0);
                    return;
                }
            }
            reply.error(libc::ENOENT);
            return;
        }

        if parent == VIEWS_INODE {
            if let Ok(entries) = self.list_views_entries() {
                for (ino, kind, entry_name) in entries {
                    if entry_name == name_str {
                        let attr = match kind {
                            FileType::Directory => self.dir_attr(ino),
                            _ => self.dir_attr(ino),
                        };
                        reply.entry(&TTL, &attr, 0);
                        return;
                    }
                }
            }
            reply.error(libc::ENOENT);
            return;
        }

        // View directories
        let view = match self.view_for_inode(parent) {
            Ok(Some(v)) => v,
            _ => {
                reply.error(libc::ENOENT);
                return;
            }
        };

        // Object entries are UUIDs
        let object_id = match uuid::Uuid::parse_str(name_str) {
            Ok(uuid) => ObjectID::from_uuid(uuid),
            Err(_) => {
                reply.error(libc::ENOENT);
                return;
            }
        };

        match self.object_in_view(&view, &object_id) {
            Ok(true) => match self.inode_for_object(&object_id) {
                Ok(ino) => match self.file_attr(&object_id, ino) {
                    Ok(attr) => reply.entry(&TTL, &attr, 0),
                    Err(_) => reply.error(libc::ENOENT),
                },
                Err(_) => reply.error(libc::ENOENT),
            },
            Ok(false) => reply.error(libc::ENOENT),
            Err(_) => reply.error(libc::ENOENT),
        }
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, reply: ReplyAttr) {
        if ino == ROOT_INODE || ino == VIEWS_INODE || ino == PROJECTS_INODE || ino == RECENT_INODE {
            reply.attr(&TTL, &self.dir_attr(ino));
            return;
        }

        if let Ok(Some(_view)) = self.view_for_inode(ino) {
            reply.attr(&TTL, &self.dir_attr(ino));
            return;
        }

        // Object file
        match self.object_id_for_inode(ino) {
            Ok(Some(object_id)) => match self.file_attr(&object_id, ino) {
                Ok(attr) => reply.attr(&TTL, &attr),
                Err(_) => reply.error(libc::ENOENT),
            },
            _ => reply.error(libc::ENOENT),
        }
    }

    fn readdir(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        let mut entries: Vec<(u64, FileType, String)> = Vec::new();

        if ino == ROOT_INODE {
            entries.push((ROOT_INODE, FileType::Directory, ".".to_string()));
            entries.push((ROOT_INODE, FileType::Directory, "..".to_string()));
            entries.extend(self.list_root_entries());
        } else if ino == VIEWS_INODE {
            entries.push((VIEWS_INODE, FileType::Directory, ".".to_string()));
            entries.push((ROOT_INODE, FileType::Directory, "..".to_string()));
            match self.list_views_entries() {
                Ok(mut view_entries) => entries.append(&mut view_entries),
                Err(_) => {
                    reply.error(libc::ENOENT);
                    return;
                }
            }
        } else if let Ok(Some(view)) = self.view_for_inode(ino) {
            entries.push((ino, FileType::Directory, ".".to_string()));
            let parent = if ino == PROJECTS_INODE || ino == RECENT_INODE {
                ROOT_INODE
            } else {
                VIEWS_INODE
            };
            entries.push((parent, FileType::Directory, "..".to_string()));

            match self.resolve_view_objects(&view) {
                Ok(objects) => {
                    for object_id in objects {
                        if let Ok(obj_ino) = self.inode_for_object(&object_id) {
                            entries.push((obj_ino, FileType::RegularFile, object_id.to_string()));
                        }
                    }
                }
                Err(_) => {
                    reply.error(libc::ENOENT);
                    return;
                }
            }
        } else {
            reply.error(libc::ENOENT);
            return;
        }

        for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
            let (ino, kind, name) = entry;
            if reply.add(ino, (i + 1) as i64, kind, name) {
                break;
            }
        }
        reply.ok();
    }

    fn open(&mut self, _req: &Request<'_>, ino: u64, flags: i32, reply: ReplyOpen) {
        let _ = (ino, flags);
        reply.opened(0, 0);
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
        let object_id = match self.object_id_for_inode(ino) {
            Ok(Some(id)) => id,
            _ => {
                reply.error(libc::ENOENT);
                return;
            }
        };

        let object = match self.repo.metadata.load_object(&object_id) {
            Ok(obj) => obj,
            Err(_) => {
                reply.error(libc::ENOENT);
                return;
            }
        };

        if self
            .repo
            .authorize_object_permission(&object, crate::crypto::Permission::Read, false)
            .is_err()
        {
            reply.error(libc::EACCES);
            return;
        }

        if crate::security::is_quarantined_executable(&object.tags) {
            reply.error(libc::EACCES);
            return;
        }

        let version = match self.repo.metadata.load_version(&object.current_version) {
            Ok(v) => v,
            Err(_) => {
                reply.error(libc::ENOENT);
                return;
            }
        };

        let manifest = match self.repo.metadata.load_manifest(&version.manifest_ref) {
            Ok(m) => m,
            Err(_) => {
                reply.error(libc::ENOENT);
                return;
            }
        };

        let data = match self
            .repo
            .chunks
            .read_range_sync(&manifest, offset as u64, size)
        {
            Ok(d) => d,
            Err(_) => {
                reply.error(libc::EIO);
                return;
            }
        };

        reply.data(&data);
    }
}

fn micros_to_systemtime(micros: i64) -> SystemTime {
    if micros <= 0 {
        return SystemTime::UNIX_EPOCH;
    }
    SystemTime::UNIX_EPOCH + Duration::from_micros(micros as u64)
}
