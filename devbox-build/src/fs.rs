use std::io::Write;
use std::ffi::OsStr;
use std::marker::PhantomData;
use std::ops::Add;
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;

use globset::{ GlobBuilder, GlobMatcher };

use super::Resource;
use super::Set;

//-- Unit ------------------------------------------------------------------------------------------

/// Enum based Resource that is either a File or a Dir
///
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Unit {
    Dir(Dir),
    File(File),
}

impl Unit {

    /// Delegates to File or Dir path() mathod
    //TODO: test
    pub fn path(&self) -> &Path {
        match self {
           Unit::Dir(ref res) => res.path(),
           Unit::File(ref res) => res.path(),
        }
    }

    /// Delegates to File or Dir link_from_inside() mathod
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

//-- File ------------------------------------------------------------------------------------------

/// Resource representing file system file
///
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct File {
    path: PathBuf
}

impl File {

    /// Create new File pointing to absolute file system `path`
    pub fn new<P:AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        match normalize(path.as_ref()) {
            Some(path) if path.is_absolute() => Ok(File { path }),
            _ => Err(format!("Path {0} is not absolute", path.as_ref().display()).into())
        }
    }

    /// Path reference to file system file
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Creates the file using [`create`](#method.create) and returns itself or stops the build with
    /// informative error message.
    pub fn created(self) -> Self {
        self.create();
        self
    }

    /// Creates the file using [`create_result`](#method.create_result) or stops the build with
    /// informative error message.
    pub fn create(&self) -> std::fs::File {
        self.create_result().expect(format!("Creating file {} FAILED", self).as_str())
    }

    /// Creates (or truncates) the file and any missing directories on it's path in write only mode.
    pub fn create_result(&self) -> std::io::Result<std::fs::File> {
        println!("Creating file: {}", self);

        if let Some(parent) = self.parent() {
            parent.create_result()?;
        }

        std::fs::File::create(&self.path)
    }

    /// Creating a link to this file from another directory with this file's name returning self
    /// or stopping the build with informative error message.
    ///
    /// If the directory already contains an entry with this name, linking fails.
    pub fn linked_from_inside(self, dir: &Dir) -> Self {
        dir.file(self.path().file_name().unwrap()).link_to(&self);
        self
    }

    /// Creating a link to this file from another directory with this file's name or stops the build
    /// with informative error message.
    ///
    /// If the directory already contains an entry with this name, linking fails.
    pub fn link_from_inside(&self, dir: &Dir) {
        dir.file(self.path().file_name().unwrap()).link_to(self);
    }

    /// Creating a link to this file from another directory with this file's name.
    ///
    /// If the directory already contains a file or directory by this name, linking fails.
    /// To allow overwriting existing link with different target set `force` to `true` or linking to
    /// this file will also fail.
    pub fn link_from_inside_result(&self, dir: &Dir, force: bool) -> std::io::Result<()> {
        dir.file(self.path().file_name().unwrap()).link_to_result(self, force)
    }

    /// Create a symbolic link at this file path to given target file `to` creating any needed
    /// directories in the process returning self or stops the build with informative error message.
    ///
    /// If a file or directory by that name already exists, linking will fail.
    pub fn linked_to(self, to: &File) -> Self {
        self.link_to(to);
        self
    }

    /// Create a symbolic link at this file path to given target file `to` creating any needed
    /// directories in the process or stops the build with informative error message.
    ///
    /// If a file or directory by that name already exists, linking will fail.
    pub fn link_to(&self, to: &File) {
        self.link_to_result(to, false)
            .expect(format!("Creating link {} -> {} FAILED", self, to).as_str())
    }

    /// Create a symbolic link at this file path to given target file `to` creating any needed
    /// directories in the process.
    ///
    /// If a file or directory by that name already exists, linking will fail.
    /// To allow overwriting existing link to a different file set `force` to `true` or linking to
    /// a different file will also fail.
    pub fn link_to_result(&self, to: &File, force: bool) -> std::io::Result<()> {
        println!("Creating link {} -> {}", self, to);

        if let Some(parent) = self.parent() {
            parent.create_result()?;
        }

        if self.path.exists() {
            match std::fs::read_link(&self.path) {
                Ok(target) if target != to.path && force => std::fs::remove_file(self.path())?,
                Ok(target) if target == to.path => return Ok(()),
                _ => return Err(std::io::ErrorKind::AlreadyExists.into()),
            }
        }

        File::platform_make_link(&to.path, &self.path)
    }

    /// Opens file's metadata using [`metadata_result`](#method.metadata_result) or stops the build
    /// with informative error message.
    pub fn metadata(&self) -> std::fs::Metadata {
        self.metadata_result().expect(format!("Metatdata query {} FAILED", self).as_str())
    }

    /// Opens file metadata
    pub fn metadata_result(&self) -> std::io::Result<std::fs::Metadata> {
        std::fs::metadata(&self.path)
    }

    /// Opens the file using [`open_result`](#method.open_result) or stops the build with
    /// informative error message
    pub fn open(&self) -> std::fs::File {
        self.open_result().expect(format!("Opening file {} FAILED", self).as_str())
    }

    /// Opens the file in read only mode
    pub fn open_result(&self) -> std::io::Result<std::fs::File> {
        std::fs::File::open(&self.path)
    }

    /// Writes the entire content to the file using [`rewrite_result`](#method.rewrite_result) or
    /// stops the build with informative error message
    //TODO: test
    pub fn rewrite<P: AsRef<[u8]>>(&self, bytes: P) {
        self.rewrite_result(bytes).expect(format!("Writing text {} FAILED", self).as_str())
    }

    /// Writes the entire content to the file if it is different then the current one
    /// creating the file if needed.
    //TODO: test
    pub fn rewrite_result<P: AsRef<[u8]>>(&self, bytes: P) -> std::io::Result<()> {
        let bytes = bytes.as_ref();
        if let Ok(old) = std::fs::read(&self.path) {
            if old == bytes {
                return Ok(())
            }
        }

        self.create().write_all(bytes)
    }

    /// Touches the file using [`touch`](#method.touch) and returns itself or stops the build with
    /// informative error message
    pub fn touched(self) -> Self {
        self.touch();
        self
    }

    /// Touches the file using [`touch`](#method.touch) and returns itself or stops the build with
    /// informative error message.
    pub fn touch(&self) {
        self.touch_result().expect(format!("Touching file {} FAILED", self).as_str())
    }

    /// Touches the file by updating it's modification time or creating an empty one if it does not
    /// exists yet including any needed directories.
    pub fn touch_result(&self) -> std::io::Result<()> {
        println!("Touching file: {}", self);

        if !self.path.exists() {
            return self.create_result().map(|_|());
        }

        let now = filetime::FileTime::from_system_time(SystemTime::now());
        filetime::set_file_mtime(self.path.clone(), now)
    }

    /// Returns parent directory
    fn parent(&self) -> Option<Dir> {
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
        if let Ok(metadata) = self.metadata_result() {
            return metadata.modified().ok();
        }

        None
    }
}

impl AsRef<File> for File {
    fn as_ref(&self) -> &File {
        self
    }
}

impl AsRef<OsStr> for File {
    fn as_ref(&self) -> &OsStr {
        &self.path.as_ref()
    }
}

impl AsRef<Path> for File {
    fn as_ref(&self) -> &Path {
        &self.path.as_ref()
    }
}

impl Add<&File> for &File {
    type Output = Set<File>;

    fn add(self, rhs: &File) -> Self::Output {
        vec![self.clone(), rhs.clone()].into()
    }
}

impl Add<&Dir> for &File {
    type Output = Set<Unit>;

    fn add(self, rhs: &Dir) -> Self::Output {
        vec![Unit::File(self.clone()), Unit::Dir(rhs.clone())].into()
    }
}

impl std::fmt::Display for File {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        self.path.display().fmt(formatter)
    }
}

//-- Dir -------------------------------------------------------------------------------------------

/// Resource representing file system directory
///
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Dir {
    path: PathBuf
}

impl Dir {

    /// Create new Dir pointing to absolute file system `path` panicking if failed
    pub fn new<P:AsRef<Path>>(path: P) -> Self {
        Dir::new_safe(path).unwrap()
    }

    /// Create new Dir pointing to absolute file system `path`
    pub fn new_safe<P:AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        match normalize(path.as_ref()) {
            Some(path) if path.is_absolute() => Ok(Dir { path }),
            _ => Err(format!("Path {0} is not absolute", path.as_ref().display()).into())
        }
    }

    /// Path reference to file system directory
    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }

    /// Creates the directory using [`create`](#method.create) and returns itself or stops the build
    /// with informative error message.
    pub fn created(self) -> Self {
        self.create();
        self
    }

    /// Creates the directory using [`create_result`](#method.create_result) or stops the build with
    /// informative error message.
    pub fn create(&self) {
        self.create_result().expect(format!("Creating directory {} FAILED", self).as_str());
    }

    /// Creates the directory and any missing parent directories on it's path.
    pub fn create_result(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.path)
    }

    /// All directory content (files, directories and links) matching given `glob` file name pattern
    pub fn content<G:AsRef<str>>(&self, glob: G) -> DirContent<Unit> {
        DirContent::new(self.path.clone(), glob)
    }

    /// All subdirectories and directory links matching given `glob` file name pattern
    pub fn dirs<G:AsRef<str>>(&self, glob: G) -> DirContent<Dir> {
        DirContent::new(self.path.clone(), glob)
    }

    /// All files and file links matching given `glob` file name pattern
    pub fn files<G:AsRef<str>>(&self, glob: G) -> DirContent<File> {
        DirContent::new(self.path.clone(), glob)
    }

    /// Subdirectory at given relative `path`
    ///
    /// Will stop the build with informative error message if path is not relative.
    pub fn dir<P:AsRef<Path>>(&self, path: P) -> Dir {
        self.dir_result(path).unwrap()
    }

    /// Subdirectory at given relative `path`
    pub fn dir_result<P:AsRef<Path>>(&self, path: P) -> Result<Self, Box<dyn std::error::Error>> {
        match normalize(path.as_ref()) {
            Some(path) if path.is_relative() => Ok(Dir { path: self.path.join(path) }),
            _ => Err(format!("Path {0} is not relative", path.as_ref().display()).into())
        }
    }

    /// A file at given relative `path`
    ///
    /// Will stop the build with informative error message if path is not relative.
    pub fn file<P:AsRef<Path>>(&self, path: P) -> File {
        self.file_result(path).unwrap()
    }

    /// A file at given relative `path`
    pub fn file_result<P:AsRef<Path>>(&self, path: P) -> Result<File, Box<dyn std::error::Error>> {
        match normalize(path.as_ref()) {
            Some(path) if path.is_relative() => Ok(File { path: self.path.join(path) }),
            _ => Err(format!("Path '{0}' is not relative", path.as_ref().display()).into())
        }
    }

    /// Creating a link to this directory from another directory with this directory's name
    /// returning self or stopping the build with informative error message.
    ///
    /// If the directory already contains an entry with this name, linking fails.
    pub fn linked_from_inside(self, dir: &Dir) -> Self {
        dir.dir(self.path().file_name().unwrap()).link_to(&self);
        self
    }

    /// Creating a link to this directory from another directory with this directory's name
    /// or stopping the build with informative error message.
    ///
    /// If the directory already contains an entry with this name, linking fails.
    pub fn link_from_inside(&self, dir: &Dir) {
        dir.dir(self.path().file_name().unwrap()).link_to(self);
    }

    /// Creating a link to this directory from another directory with this directory's name.
    ///
    /// If the directory already contains a file or directory by this name, linking fails.
    /// To allow overwriting existing link with different target set `force` to `true` or linking to
    /// this directory will also fail.
    pub fn link_from_inside_result(&self, dir: &Dir, force: bool) -> std::io::Result<()> {
        dir.dir(self.path().file_name().unwrap()).link_to_result(self, force)
    }

    /// Create a symbolic link at this directory path to given target directory `to` creating any
    /// needed directories in the process returning self or stopping the build with informative
    /// error message.
    ///
    /// If a file or directory by that name already exists, linking will fail.
    pub fn linked_to(self, to: &Dir) -> Self {
        self.link_to(to);
        self
    }

    /// Create a symbolic link at this directory path to given target directory `to` creating any
    /// needed directories in the process or stopping the build with informative error message.
    ///
    /// If a file or directory by that name already exists, linking will fail.
    pub fn link_to(&self, to: &Dir) {
        self.link_to_result(to, false)
            .expect(format!("Creating link {} -> {} FAILED", self, to).as_str())
    }

    /// Create a symbolic link at this directory path to given target directory `to` creating any
    /// needed directories in the process.
    ///
    /// If a file or directory by that name already exists, linking will fail.
    /// To allow overwriting existing link to a different directory set `force` to `true` or linking
    /// to a different directory will also fail.
    pub fn link_to_result(&self, to: &Dir, force: bool) -> std::io::Result<()> {
        println!("Creating link {} -> {}", self, to);

        if let Some(parent) = self.parent() {
            parent.create_result()?;
        }

        if self.path.exists() {

            match std::fs::read_link(&self.path) {
                Ok(target) if target != to.path && force => std::fs::remove_file(self.path())?,
                Ok(target) if target == to.path => return Ok(()),
                _ => return Err(std::io::ErrorKind::AlreadyExists.into()),
            }
        }

        Dir::platform_make_link(&to.path, &self.path)
    }

    /// Touches the directory using [`touch`](#method.touch) and returns itself or stops the build
    /// with informative error message
    pub fn touched(self) -> Self {
        self.touch();
        self
    }

    /// Touches the directory using [`touch`](#method.touch) and returns itself or stops the build
    /// with informative error message.
    pub fn touch(&self) {
        self.touch_result().expect(format!("Touching dir {} FAILED", self).as_str())
    }

    /// Touches the directory by updating it's modification time or creating a new one if it does
    /// not exists yet including any needed directories.
    pub fn touch_result(&self) -> std::io::Result<()> {
        println!("Touching dir: {}", self);

        if !self.path.exists() {
            return self.create_result();
        }

        let now = filetime::FileTime::from_system_time(SystemTime::now());
        filetime::set_file_mtime(self.path.clone(), now)
    }

    /// Returns parent directory
    fn parent(&self) -> Option<Dir> {
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

impl AsRef<Dir> for Dir {
    fn as_ref(&self) -> &Dir {
        self
    }
}

impl AsRef<OsStr> for Dir {
    fn as_ref(&self) -> &OsStr {
        &self.path.as_ref()
    }
}

impl AsRef<Path> for Dir {
    fn as_ref(&self) -> &Path {
        &self.path.as_ref()
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
        vec![self.clone(), rhs.clone()].into()
    }
}

impl Add<&File> for &Dir {
    type Output = Set<Unit>;

    fn add(self, rhs: &File) -> Self::Output {
        vec![Unit::Dir(self.clone()), Unit::File(rhs.clone())].into()
    }
}

impl std::fmt::Display for Dir {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        self.path.display().fmt(formatter)
    }
}

//-- Path normalization ----------------------------------------------------------------------------

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

//-- DirContent ------------------------------------------------------------------------------------

/// Represents directory entries matching certain criteria like GLOB name pattern and type (files,
/// directories or units).
///
/// Matching is done on two sets of patterns:
///  - entry matches if any of the inclusion patterns matches and
///  - none of the exclusion pattern matches
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

    /// Add exlusion pattern reducing the number of matching entries
    pub fn exclude<G:AsRef<str>>(mut self, glob: G) -> Self {
        self.matchers.push(compile(false, glob));
        self
    }

    /// Add inclusion pattern increasing the number of matching entries
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

impl DirContent<Unit> {
    fn iter(&self) -> Box<dyn Iterator<Item=Unit>> {
        Box::new(self.walkdir().map(|e|
            if e.file_type().is_dir() {
                Unit::Dir( Dir { path: e.path().to_owned() })
            } else {
                Unit::File( File { path: e.path().to_owned() })
            }
        ))
    }
}

impl DirContent<Dir> {
    fn iter(&self) -> Box<dyn Iterator<Item=Dir>> {
        Box::new(self.walkdir().filter_map(|e|
            if e.file_type().is_dir() {
                Some(Dir { path: e.path().to_owned() })
            } else {
                None
            }
        ))
    }
}

impl DirContent<File> {
    fn iter(&self) -> Box<dyn Iterator<Item=File>> {
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

impl IntoIterator for DirContent<Unit> {
    type Item = Unit;
    type IntoIter = Box<dyn Iterator<Item=Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for DirContent<Dir> {
    type Item = Dir;
    type IntoIter = Box<dyn Iterator<Item=Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for DirContent<File> {
    type Item = File;
    type IntoIter = Box<dyn Iterator<Item=Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Resource for DirContent<Dir> {
    fn timestamp(&self) -> Option<SystemTime> {
        super::res::timestamp(self.iter())
    }
}

impl Resource for DirContent<File> {
    fn timestamp(&self) -> Option<SystemTime> {
        super::res::timestamp(self.iter())
    }
}

impl Resource for DirContent<Unit> {
    fn timestamp(&self) -> Option<SystemTime> {
        super::res::timestamp(self.iter())
    }
}