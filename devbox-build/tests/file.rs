use std::io::{Read, Write};
use devbox_build::*;
use devbox_test_args::args;

fn file_fix() -> (tempfile::TempDir, Dir, File) {
    let temp = tempfile::tempdir().unwrap();
    let root = Dir::new(temp.path());
    let file = root.file("nested/foo.txt");
    (temp, root, file)
}

// create ------------------------------------------------------------------------------------------

#[args(
    safe: |file:&File| { file.create_result().unwrap() };
    easy: |file:&File| { file.create() };
    bild: |file:&File| {
        file.clone().created().touched();
        std::fs::OpenOptions::new().append(true).open(file.path()).unwrap()
    }
)]
#[test]
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

#[args(
    safe: |file:&File,to| { file.link_to_result(to, false).unwrap(); };
    easy: |file:&File,to| { file.link_to(to); };
    bild: |file:&File,to| { file.clone().linked_to(to); }
)]
#[test]
fn file_link_to(link_to:_) {
    let (_, root, file) = file_fix();
    file.create();

    let link = root.file("nested/link");
    link_to(&link, &file);
    link_to(&link, &file);

    assert_eq!(file.path(), std::fs::read_link(link.path()).unwrap());
}

#[args(
    forc: |file:&File,to| { file.link_to_result(to, true).expect("Link"); };
    safe: |file:&File,to| { file.link_to_result(to, false).expect("Link"); } ! "Link";
    easy: |file:&File,to| { file.link_to(to); }           ! "Creating link";
    bild: |file:&File,to| { file.clone().linked_to(to); } ! "Creating link"
)]
#[test]
fn file_link_to_overwrite_link(link_to:_) {
    let (_, root, file) = file_fix();
    file.create();
    let foe = root.file("foe");

    let link = root.file("nested/link");
    link_to(&link, &file);
    link_to(&link, &foe);
}

#[args(
    forc: |file:&File,to| { file.link_to_result(to, true).expect("Link"); } ! "Link";
    safe: |file:&File,to| { file.link_to_result(to, false).expect("Link"); } ! "Link";
    easy: |file:&File,to| { file.link_to(to); }           ! "Creating link";
    bild: |file:&File,to| { file.clone().linked_to(to); } ! "Creating link"
)]
#[test]
fn file_link_to_overwrite_file(link_to:_) {
    let (_, root, file) = file_fix();
    file.create();

    let link = root.file("nested/link");
    link.create();
    link_to(&link, &file);
}

// link_from_inside --------------------------------------------------------------------------------

#[args(
    safe: |file:&File,from| { file.link_from_inside_result(from, false).unwrap(); };
    easy: |file:&File,from| { file.link_from_inside(from); };
    bild: |file:&File,from| { file.clone().linked_from_inside(from); };
)]
#[test]
fn file_link_from_inside(link_from_inside:_) {
    let (_, root, target) = file_fix();
    let dir = root.dir("from");
    target.create();

    link_from_inside(&target, &dir);
    link_from_inside(&target, &dir);

    assert_eq!(target.path(), std::fs::read_link(dir.dir("foo.txt").path()).unwrap());
}

#[args(
    forc: |file:&File,from| { file.link_from_inside_result(from, true).expect("Creating link"); } ! "Creating link";
    safe: |file:&File,from| { file.link_from_inside_result(from, false).expect("Creating link"); } ! "Creating link";
    easy: |file:&File,from| { file.link_from_inside(from); } ! "Creating link";
    bild: |file:&File,from| { file.clone().linked_from_inside(from) } ! "Creating link";
)]
#[test]
fn file_link_from_inside_overwrite_file(link_from_inside:_) {
    let (_, root, target) = file_fix();
    let dir = root.dir("from");
    target.create();

    dir.file("foo.txt").create();
    link_from_inside(&target, &dir);
}

#[args(
    forc: |file:&File,from| { file.link_from_inside_result(from, true).expect("Link"); };
    safe: |file:&File,from| { file.link_from_inside_result(from, false).expect("Link"); } ! "Link";
    easy: |file:&File,from| { file.link_from_inside(from); } ! "Creating link";
    bild: |file:&File,from| { file.clone().linked_from_inside(from) } ! "Creating link";
)]
#[test]
fn file_link_from_inside_overwrite_link(link_from_inside:_) {
    let (_, root, target) = file_fix();
    let dir = root.dir("from");
    let another = root.file("foo.txt").touched();
    target.create();

    another.link_from_inside(&dir);
    link_from_inside(&target, &dir);
}

// metadata ----------------------------------------------------------------------------------------

#[args(
    safe: |file:&File| { file.metadata_result().unwrap() };
    easy: |file:&File| { file.metadata() }
)]
#[test]
fn file_metadata(metadata:_) {
    let (_, _, file) = file_fix();
    file.create();

    let actual = metadata(&file);
    let expect = std::fs::metadata(file.path()).unwrap();
    assert_eq!(expect.len(), actual.len());
    assert_eq!(expect.modified().unwrap(), actual.modified().unwrap());
}

#[args(
    safe: |file:&File| { file.metadata_result().expect("Metatdata query") } ! "Metatdata query";
    easy: |file:&File| { file.metadata() } ! "Metatdata query"
)]
#[test]
fn file_metadata_nonexistent(metadata:_) {
    let (_, _, file) = file_fix();
    metadata(&file);
}

// open --------------------------------------------------------------------------------------------

#[args(
    safe: |file:&File| { file.open_result().unwrap() };
    easy: |file:&File| { file.open() }
)]
#[test]
fn file_open(open:_) {
    let (_, _, file) = file_fix();
    file.create().write_all(b"foo").unwrap();

    let mut io = open(&file);
    assert_eq!("foo", { let mut b = String::new(); io.read_to_string(&mut b).unwrap(); b});
}

#[args(
    safe: |file:&File| { file.open_result().expect("Opening file") } ! "Opening file";
    easy: |file:&File| { file.open() } ! "Opening file"
)]
#[test]
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
    assert_eq!(std::fs::metadata(file.path()).unwrap().modified().unwrap(),
               file.timestamp().unwrap());
}

// touch -------------------------------------------------------------------------------------------

#[args(
    safe: |file:&File| { file.touch_result().unwrap(); };
    easy: |file:&File| { file.touch(); };
    bild: |file:&File| { file.clone().touched(); }
)]
#[test]
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
    let baz = Dir::new("/a").file("baz");
    let mut foobar = (&foo + &bar + &baz).into_iter();
    assert_eq!(Some(foo), foobar.next());
    assert_eq!(Some(bar), foobar.next());
    assert_eq!(Some(baz), foobar.next());
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