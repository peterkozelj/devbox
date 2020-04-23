use std::fs;
use std::marker::PhantomData;
use std::ops::Add;
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;

use globset::{ GlobBuilder, GlobMatcher };

use super::Resource;
use super::Set;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Unit
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Unit {
    Dir(Dir),
    File(File),
}

impl Unit {
    //TODO: test
    pub fn path(&self) -> &Path {
        match self {
           Unit::Dir(ref res) => res.path(),
           Unit::File(ref res) => res.path(),
        }
    }

    //TODO: test
    pub fn link_from_inside(&self, dir: &Dir) {
        match self {
           Unit::Dir(ref res) => res.link_from_inside(dir),
           Unit::File(ref res) => res.link_from_inside(dir),
        }
    }
}

impl Resource for Unit {
    //TODO: test
    fn timestamp(&self) -> Option<SystemTime> {
        match self {
           Unit::Dir(ref res) => res.timestamp(),
           Unit::File(ref res) => res.timestamp(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// File
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct File {
    path: PathBuf
}

impl File {

    pub fn new(path: PathBuf) -> Self {
        File {
            path: path
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn created(self) -> Self {
        self.create();
        self
    }

    pub fn create(&self) -> std::fs::File {
        self.create_safe().expect(format!("Creating file {} FAILED", self).as_str())
    }

    pub fn create_safe(&self) -> std::io::Result<std::fs::File> {
        println!("Creating file: {}", self);

        if let Some(parent) = self.parent_safe() {
            parent.create_safe()?;
        }

        std::fs::File::create(&self.path)
    }

    pub fn open(&self) -> std::fs::File {
        self.open_safe().expect(format!("Opening file {} FAILED", self).as_str())
    }

    pub fn open_safe(&self) -> std::io::Result<std::fs::File> {
        std::fs::File::open(&self.path)
    }

    pub fn metadata(&self) -> std::fs::Metadata {
        self.metadata_safe().expect(format!("Metatdata query {} FAILED", self).as_str())
    }

    pub fn metadata_safe(&self) -> std::io::Result<std::fs::Metadata> {
        std::fs::metadata(&self.path)
    }

    pub fn touched(self) -> Self {
        self.touch();
        self
    }

    pub fn touch(&self) {
        self.touch_safe().expect(format!("Touching file {} FAILED", self).as_str())
    }

    pub fn touch_safe(&self) -> std::io::Result<()> {
        println!("Touching file: {}", self);

        if !self.path.exists() {
            return self.create_safe().map(|_|());
        }

        let now = filetime::FileTime::from_system_time(SystemTime::now());
        filetime::set_file_mtime(self.path.clone(), now)
    }

    pub fn linked_from_inside(self, dir: &Dir) -> Self {
        dir.file(self.path().file_name().unwrap()).link_to(&self);
        self
    }

    pub fn link_from_inside(&self, dir: &Dir) {
        dir.file(self.path().file_name().unwrap()).link_to(self);
    }

    pub fn link_from_inside_safe(&self, dir: &Dir) -> std::io::Result<()> {
        dir.file(self.path().file_name().unwrap()).link_to_safe(self)
    }

    pub fn linked_to(self, to: &File) -> Self {
        self.link_to(to);
        self
    }

    pub fn link_to(&self, to: &File) {
        self.link_to_safe(to).expect(format!("Creating link {} -> {} FAILED", self, to).as_str())
    }

    pub fn link_to_safe(&self, to: &File) -> std::io::Result<()> {
        println!("Creating link {} -> {}", self, to);

        if let Some(parent) = self.parent_safe() {
            parent.create_safe()?;
        }

        if self.path.exists() {
            return match std::fs::read_link(&self.path) {
                Ok(target) if target == to.path => Ok(()),
                _ => Err(std::io::ErrorKind::AlreadyExists.into()),
            }
        }

        File::platform_make_link(&to.path, &self.path)
    }

    fn parent_safe(&self) -> Option<Dir> {
        self.path.parent().map(|parent| Dir { path: parent.to_owned() })
    }

    #[cfg(not(windows))]
    fn platform_make_link<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> std::io::Result<()> {
        std::os::unix::fs::symlink(src, dst)
    }

    #[cfg(windows)]
    fn platform_make_link<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> std::io::Result<()> {
        std::os::windows::fs::symlink_file(src, dst)
    }
}

impl Resource for File {

   fn timestamp(&self) -> Option<SystemTime> {
        if let Ok(metadata) = self.metadata_safe() {
            return metadata.modified().ok();
        }

        None
    }
}

impl Add<&File> for &File {
    type Output = Set<File>;

    fn add(self, rhs: &File) -> Self::Output {
        Set { items: vec![self.clone(), rhs.clone()] }
    }
}

impl Add<&Dir> for &File {
    type Output = Set<Unit>;

    fn add(self, rhs: &Dir) -> Self::Output {
        Set { items: vec![Unit::File(self.clone()), Unit::Dir(rhs.clone())] }
    }
}

impl std::fmt::Display for File {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        self.path.display().fmt(formatter)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Dir
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Dir {
    path: PathBuf
}

impl Dir {

    pub fn new<P:AsRef<Path>>(path: P) -> Self {
        Dir::new_safe(path).unwrap()
    }

    pub fn new_safe<P:AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        match normalize(path.as_ref()) {
            Some(path) if path.is_absolute() => Ok(Dir { path }),
            _ => Err(format!("Path {0} is not absolute", path.as_ref().display()).into())
        }
    }

    pub fn content<G:AsRef<str>>(&self, glob: G) -> DirContent<Unit> {
        DirContent::new(self.path.clone(), glob)
    }

    pub fn dirs<G:AsRef<str>>(&self, glob: G) -> DirContent<Dir> {
        DirContent::new(self.path.clone(), glob)
    }

    pub fn files<G:AsRef<str>>(&self, glob: G) -> DirContent<File> {
        DirContent::new(self.path.clone(), glob)
    }

    pub fn dir<P:AsRef<Path>>(&self, path: P) -> Dir {
        self.dir_safe(path).unwrap()
    }

    pub fn dir_safe<P:AsRef<Path>>(&self, path: P) -> Result<Self, Box<dyn std::error::Error>> {
        match normalize(path.as_ref()) {
            Some(path) if path.is_relative() => Ok(Dir { path: self.path.join(path) }),
            _ => Err(format!("Path {0} is not relative", path.as_ref().display()).into())
        }
    }

    pub fn file<P:AsRef<Path>>(&self, path: P) -> File {
        self.file_safe(path).unwrap()
    }

    pub fn file_safe<P:AsRef<Path>>(&self, path: P) -> Result<File, Box<dyn std::error::Error>> {
        match normalize(path.as_ref()) {
            Some(path) if path.is_relative() => Ok(File { path: self.path.join(path) }),
            _ => Err(format!("Path '{0}' is not relative", path.as_ref().display()).into())
        }
    }

    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }

    pub fn created(self) -> Self {
        self.create();
        self
    }

    pub fn create(&self) {
        self.create_safe().expect(format!("Could not make directory {}", self).as_str());
    }

    pub fn create_safe(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.path)
    }

    pub fn touched(self) -> Self {
        self.touch();
        self
    }

    pub fn touch(&self) {
        self.touch_safe().expect(format!("Touching dir {} FAILED", self).as_str())
    }

    pub fn touch_safe(&self) -> std::io::Result<()> {
        println!("Touching dir: {}", self);

        if !self.path.exists() {
            return self.create_safe();
        }

        let now = filetime::FileTime::from_system_time(SystemTime::now());
        filetime::set_file_mtime(self.path.clone(), now)
    }

    pub fn linked_from_inside(self, dir: &Dir) -> Self {
        dir.dir(self.path().file_name().unwrap()).link_to(&self);
        self
    }

    pub fn link_from_inside(&self, dir: &Dir) {
        dir.dir(self.path().file_name().unwrap()).link_to(self);
    }

    pub fn link_from_inside_safe(&self, dir: &Dir) -> std::io::Result<()> {
        dir.dir(self.path().file_name().unwrap()).link_to_safe(self)
    }

    pub fn linked_to(self, to: &Dir) -> Self {
        self.link_to(to);
        self
    }

    pub fn link_to(&self, to: &Dir) {
        self.link_to_safe(to).expect(format!("Creating link {} -> {} FAILED", self, to).as_str())
    }

    pub fn link_to_safe(&self, to: &Dir) -> std::io::Result<()> {
        println!("Creating link {} -> {}", self, to);

        if let Some(parent) = self.parent_safe() {
            parent.create_safe()?;
        }

        if self.path.exists() {
            return match std::fs::read_link(&self.path) {
                Ok(target) if target == to.path => Ok(()),
                _ => Err(std::io::ErrorKind::AlreadyExists.into()),
            }
        }

        Dir::platform_make_link(&to.path, &self.path)
    }

    fn parent_safe(&self) -> Option<Dir> {
        self.path.parent().map(|parent| Dir { path: parent.to_owned() })
    }

    #[cfg(not(windows))]
    fn platform_make_link<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> std::io::Result<()> {
        std::os::unix::fs::symlink(src, dst)
    }

    #[cfg(windows)]
    fn platform_make_link<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> std::io::Result<()> {
        std::os::windows::fs::symlink_dir(src, dst)
    }
}

fn normalize<P:AsRef<Path>>(subpath: P) -> Option<PathBuf> {
    let result = PathBuf::from(subpath.as_ref()).components().fold(Some((PathBuf::new(), 0)), |r, c| {
        match r {
            None => None,
            Some((mut path, depth)) => match c {
                Component::Normal(value) => {
                    path.push(value);
                    Some((path, depth + 1))
                }
                Component::CurDir => {
                    Some((path, depth))
                }
                Component::ParentDir => {
                    if depth == 0 {
                        return  None
                    }
                    path.pop();
                    Some((path, depth - 1))
                }
                Component::RootDir => {
                    path.push("/");
                    Some((path, 0))
                }
                Component::Prefix(prefix) => {
                    path.push(prefix.as_os_str());
                    Some((path, -1))
                }
            }
        }
    });

    if let Some((path,depth)) = result {
        if depth > 0 {
            return Some(path)
        }
    }

    return None;
}

impl AsRef<Dir> for Dir {
    fn as_ref(&self) -> &Dir {
        self
    }
}

impl Resource for Dir {
    fn timestamp(&self) -> Option<SystemTime> {
        if let Ok(metadata) = std::fs::metadata(&self.path) {
            return metadata.modified().ok();
        }

        None
    }
}

impl Add<&Dir> for &Dir {
    type Output = Set<Dir>;

    fn add(self, rhs: &Dir) -> Self::Output {
        Set { items: vec![self.clone(), rhs.clone()] }
    }
}

impl Add<&File> for &Dir {
    type Output = Set<Unit>;

    fn add(self, rhs: &File) -> Self::Output {
        Set { items: vec![Unit::Dir(self.clone()), Unit::File(rhs.clone())] }
    }
}

impl std::fmt::Display for Dir {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        self.path.display().fmt(formatter)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// DirContent
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct DirContent<T> {
    path: PathBuf,
    matchers: Vec<(GlobMatcher, bool)>,
    phantom: PhantomData<T>,
}

impl<T> DirContent<T> {

    fn new<G:AsRef<str>>(path: PathBuf, glob: G) -> Self {
        DirContent {
            phantom: PhantomData,
            path,
            matchers: vec![compile(true, glob)],
        }
    }

    pub fn exclude<G:AsRef<str>>(mut self, glob: G) -> Self {
        self.matchers.push(compile(false, glob));
        self
    }

    pub fn include<G:AsRef<str>>(mut self, glob: G) -> Self {
        self.matchers.push(compile(true, glob));
        self
    }

    fn walkdir(&self) -> impl Iterator<Item=walkdir::DirEntry> {
        let root = self.path.clone();
        let matchers = self.matchers.clone();
        walkdir::WalkDir::new(&self.path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(move |e| e.depth() > 0 && {
                let relative = e.path().strip_prefix(&root).unwrap();
                let mut matched = false;
                for matcher in &matchers {
                    if matcher.0.is_match(relative) {
                        matched = matcher.1 || return false;
                    }
                }
                matched
            })
    }
}

fn compile<G:AsRef<str>>(incl: bool, glob: G) -> (GlobMatcher, bool) {
    (
        GlobBuilder::new(glob.as_ref()).literal_separator(true).build().unwrap().compile_matcher(),
        incl
    )
}

impl<'a> IntoIterator for DirContent<Unit> {
    type Item = Unit;
    type IntoIter = Box<dyn Iterator<Item=Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.walkdir().map(|e|
            if e.file_type().is_dir() {
                Unit::Dir( Dir { path: e.path().to_owned() })
            } else {
                Unit::File( File { path: e.path().to_owned() })
            }
        ))
    }
}

impl IntoIterator for DirContent<Dir> {
    type Item = Dir;
    type IntoIter = Box<dyn Iterator<Item=Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.walkdir().filter_map(|e|
            if e.file_type().is_dir() {
                Some(Dir { path: e.path().to_owned() })
            } else {
                None
            }
        ))
    }
}

impl IntoIterator for DirContent<File> {
    type Item = File;
    type IntoIter = Box<dyn Iterator<Item=Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.walkdir().filter_map(|e|
            if e.file_type().is_file() {
                Some(File { path: e.path().to_owned() })
            } else {
                None
            }
        ))
    }
}

impl<T> AsRef<DirContent<T>> for DirContent<T> {
    fn as_ref(&self) -> &DirContent<T> {
        self
    }
}