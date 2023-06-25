use core::fmt::Display;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::ops::Deref;

#[derive(Debug, Clone)]
pub struct VfsPath {
    from_root: bool,
    inner: Vec<String>,
}

impl VfsPath {
    pub fn empty(from_root: bool) -> VfsPath {
        VfsPath {
            from_root,
            inner: Vec::new(),
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.inner.iter().map(|x| x.as_str())
    }

    pub fn is_from_root(&self) -> bool {
        self.from_root
    }

    pub fn push(&mut self, next: &str) {
        self.inner.push(next.to_string());
    }

    pub fn parent(&self) -> Self {
        if self.is_from_root() {
            let mut new_inner = self.inner.clone();
            new_inner.pop();
            Self {
                from_root: true,
                inner: new_inner,
            }
        } else {
            todo!()
        }
    }
}

// 实现 display 也实现了 to_string
impl Display for VfsPath {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let joined = self.inner.join("/");
        if self.from_root {
            write!(f, "/{}", joined)
        } else {
            write!(f, "{}", joined)
        }
    }
}

impl Deref for VfsPath {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<&str> for VfsPath {
    fn from(path: &str) -> Self {
        let from_root = path.starts_with('/');

        Self {
            from_root,
            inner: path
                .split('/')
                .filter(|mid| !mid.is_empty())
                .map(String::from)
                .collect(),
        }
    }
}

// 将 &VfsPath 转为 String
impl From<&VfsPath> for String {
    fn from(val: &VfsPath) -> Self {
        val.to_string()
    }
}
