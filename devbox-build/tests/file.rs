use std::io::{Read, Write};
use devbox_build::*;
mod devbox {
    pub use devbox_test::test;
}

fn file_fix() -> (tempfile::TempDir, Dir, File) {
    let temp = tempfile::tempdir().unwrap();
    let root = Dir::new(temp.path());
    let file = root.file("nested/foo.txt");
    (temp, root, file)
}

// create ------------------------------------------------------------------------------------------

#[devbox::test(
    safe: |file:&File| { file.create_safe().unwrap() };
    easy: |file:&File| { file.create() };
    bild: |file:&File| {
        file.clone().created().touched();
        std::fs::OpenOptions::new().append(true).open(file.path()).unwrap()
    }
)]
fn file_create(create:_) {
    let (_, _, file) = file_fix();

    //create
    create(&file).write_all(b"foobar").unwrap();
    assert_eq!("foobar", std::fs::read_to_string(file.path()).unwrap());

    //create-rewrite
    create(&file);
    assert_eq!("", std::fs::read_to_string(file.path()).unwrap());
}

// link_to -----------------------------------------------------------------------------------------

#[devbox::test(
    safe: |file:&File,to| { file.link_to_safe(to).unwrap(); };
    easy: |file:&File,to| { file.link_to(to); };
    bild: |file:&File,to| { file.clone().linked_to(to); }
)]
fn file_link_to(link_to:_) {
    let (_, root, file) = file_fix();
    file.create();

    let link = root.file("nested/link");
    link_to(&link, &file);
    link_to(&link, &file);

    assert_eq!(file.path(), std::fs::read_link(link.path()).unwrap());
}

#[devbox::test(
    safe: |file:&File,to| { file.link_to_safe(to).expect("Creating link"); } ! "Creating link";
    easy: |file:&File,to| { file.link_to(to); }           ! "Creating link";
    bild: |file:&File,to| { file.clone().linked_to(to); } ! "Creating link"
)]
fn file_link_to_overwrite_link(link_to:_) {
    let (_, root, file) = file_fix();
    file.create();
    let foe = root.file("foe");

    let link = root.file("nested/link");
    link_to(&link, &file);
    link_to(&link, &foe);
}

#[devbox::test(
    safe: |file:&File,to| { file.link_to_safe(to).expect("Creating link"); } ! "Creating link";
    easy: |file:&File,to| { file.link_to(to); }           ! "Creating link";
    bild: |file:&File,to| { file.clone().linked_to(to); } ! "Creating link"
)]
fn file_link_to_overwrite_file(link_to:_) {
    let (_, root, file) = file_fix();
    file.create();

    let link = root.file("nested/link");
    link.create();
    link_to(&link, &file);
}

// link_from_inside --------------------------------------------------------------------------------

#[devbox::test(
    safe: |file:&File,from| { file.link_from_inside_safe(from).unwrap(); };
    easy: |file:&File,from| { file.link_from_inside(from); };
    bild: |file:&File,from| { file.clone().linked_from_inside(from); };
)]
fn file_link_from_inside(link_from_inside:_) {
    let (_, root, target) = file_fix();
    let dir = root.dir("from");
    target.create();

    link_from_inside(&target, &dir);
    link_from_inside(&target, &dir);

    assert_eq!(target.path(), std::fs::read_link(dir.dir("foo.txt").path()).unwrap());
}

#[devbox::test(
    safe: |file:&File,from| { file.link_from_inside_safe(from).expect("Creating link"); } ! "Creating link";
    easy: |file:&File,from| { file.link_from_inside(from); } ! "Creating link";
    bild: |file:&File,from| { file.clone().linked_from_inside(from) } ! "Creating link";
)]
fn file_link_from_inside_overwrite(link_from_inside:_) {
    let (_, root, target) = file_fix();
    let dir = root.dir("from");
    target.create();

    dir.file("foo.txt").create();
    link_from_inside(&target, &dir);
}

// metadata ----------------------------------------------------------------------------------------

#[devbox::test(
    safe: |file:&File| { file.metadata_safe().unwrap() };
    easy: |file:&File| { file.metadata() }
)]
fn file_metadata(metadata:_) {
    let (_, _, file) = file_fix();
    file.create();

    let actual = metadata(&file);
    let expect = std::fs::metadata(file.path()).unwrap();
    assert_eq!(expect.len(), actual.len());
    assert_eq!(expect.modified().unwrap(), actual.modified().unwrap());
}

#[devbox::test(
    safe: |file:&File| { file.metadata_safe().expect("Metatdata query") } ! "Metatdata query";
    easy: |file:&File| { file.metadata() } ! "Metatdata query"
)]
fn file_metadata_nonexistent(metadata:_) {
    let (_, _, file) = file_fix();
    metadata(&file);
}

// open --------------------------------------------------------------------------------------------

#[devbox::test(
    safe: |file:&File| { file.open_safe().unwrap() };
    easy: |file:&File| { file.open() }
)]
fn file_open(open:_) {
    let (_, _, file) = file_fix();
    file.create().write_all(b"foo").unwrap();

    let mut io = open(&file);
    assert_eq!("foo", { let mut b = String::new(); io.read_to_string(&mut b).unwrap(); b});
}

#[devbox::test(
    safe: |file:&File| { file.open_safe().expect("Opening file") } ! "Opening file";
    easy: |file:&File| { file.open() } ! "Opening file"
)]
fn file_open_nonexistent(open:_) {
    let (_, _, file) = file_fix();
     open(&file);
}

// timestamp ---------------------------------------------------------------------------------------

#[test]
fn file_timestamp() {
    let (_, _, file) = file_fix();
    assert_eq!(None, file.timestamp());

    file.create();
    assert_eq!(std::fs::metadata(file.path()).unwrap().modified().unwrap(), file.timestamp().unwrap());
}

// touch -------------------------------------------------------------------------------------------

#[devbox::test(
    safe: |file:&File| { file.touch_safe().unwrap(); };
    easy: |file:&File| { file.touch(); };
    bild: |file:&File| { file.clone().touched(); }
)]
fn file_touch(touch:_) {
    let (_, _, file) = file_fix();

    //touch-create
    touch(&file);
    assert!(file.path().exists());

    //touch-modify
    file.create().write_all(b"foo").unwrap();
    let before = std::fs::metadata(file.path()).unwrap().modified().unwrap();

    touch(&file);
    assert_eq!("foo", std::fs::read_to_string(file.path()).unwrap());
    assert_eq!(true, before < std::fs::metadata(file.path()).unwrap().modified().unwrap());
}

// ops ---------------------------------------------------------------------------------------------

#[test]
fn file_add_file() {
    let foo = Dir::new("/a").file("foo");
    let bar = Dir::new("/a").file("bar");
    let mut foobar = (&foo + &bar).into_iter();
    assert_eq!(Some(foo), foobar.next());
    assert_eq!(Some(bar), foobar.next());
    assert_eq!(None, foobar.next());
}

#[test]
fn file_add_dir() {
    let foo = Dir::new("/a").file("foo");
    let bar = Dir::new("/a");
    let mut foobar = (&foo + &bar).into_iter();
    assert_eq!(Some(Unit::File(foo)), foobar.next());
    assert_eq!(Some(Unit::Dir(bar)), foobar.next());
    assert_eq!(None, foobar.next());
}