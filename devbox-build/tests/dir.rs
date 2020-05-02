use std::path::{PathBuf};

use devbox_build::*;
use devbox_test_args::test_args;


fn dir_fix() -> (tempfile::TempDir, Dir, Dir) {
    let temp = tempfile::tempdir().unwrap();
    let root = Dir::from(temp.path());
    let dir = root.dir("nested/foo");
    assert_eq!(false, dir.path().exists());
    (temp, root, dir)
}

// path like args ----------------------------------------------------------------------------------

#[test]
fn dir_accept_path_like() {
    let a_str_ref = "/foo";
    let a_str_buf = "/foo".to_owned();
    let a_pth_buf = PathBuf::from("/foo");
    let r_str_ref = "foo";
    let r_str_buf = "foo".to_owned();
    let r_pth_buf = PathBuf::from("foo");
    Dir::from(a_str_ref).dir(r_str_ref).file(r_str_ref);
    Dir::from(&a_str_buf).dir(&r_str_buf).file(&r_str_buf);
    Dir::from(&a_pth_buf).dir(&r_pth_buf).file(&r_pth_buf);
    Dir::from(a_str_buf.clone()).dir(r_str_buf.clone()).file(r_str_buf.clone());
    Dir::from(a_pth_buf.clone()).dir(r_pth_buf.clone()).file(r_pth_buf.clone());
}

// new ---------------------------------------------------------------------------------------------

#[test_args(
    easy: |p| Dir::from(p);
    safe: |p| Dir::new(p).unwrap()
)]
#[test_args(
    simple: "/foo/bar/baz";
    resolv: "/foo/../foo/bar/baz/x/y/../..";
    nonabs_simple: "foo/bar/baz" ! "is not absolute";
    nonabs_resolv: "/foo/../../x/foo/bar/baz" ! "is not absolute"
)]
fn dir_new(new:_, path:_) {
    assert_eq!(PathBuf::from("/foo/bar/baz"), new(path).path());
}

// dir ---------------------------------------------------------------------------------------------

#[test_args(
    easy: |p| Dir::from("/foo").dir(p);
    safe: |p| Dir::from("/foo").dir_result(p).unwrap();
)]
#[test_args(
    simple: "bar/baz";
    resolv: "bar/../bar/baz/x/y/../..";
    nonsub_empty: ""                ! "is not relative";
    nonsub_self: "bar/.."           ! "is not relative";
    nonsub_escape: "../foo/bar/baz" ! "is not relative";
    nonsub_absolute: "/foo/bar/baz" ! "is not relative";
)]
fn dir_dir(dir:_, subdir: &str) {
    assert_eq!(PathBuf::from("/foo/bar/baz"), dir(subdir).path());
}

// file --------------------------------------------------------------------------------------------

#[test_args(
    easy: |p| Dir::from("/foo").file(p);
    safe: |p| Dir::from("/foo").file_result(p).unwrap();
)]
#[test_args(
    simple: "bar/baz";
    resolve: "bar/../bar/baz/x/y/../..";
    nonsub_empty: ""                ! "is not relative";
    nonsub_self: "bar/.."           ! "is not relative";
    nonsub_escape: "../foo/bar/baz" ! "is not relative";
    nonsub_absolute: "/foo/bar/baz" ! "is not relative";
)]
fn dir_file(file:_, path: &str) {
    assert_eq!(PathBuf::from("/foo/bar/baz"), file(path).path());
}

// create ------------------------------------------------------------------------------------------

#[test_args(
    easy: |d:&Dir| d.create();
    safe: |d:&Dir| d.create_result().unwrap();
    bild: |d:&Dir| d.clone().created();
)]
fn dir_create(create:_) {
    let (_, _, dir) = dir_fix();

    create(&dir);
    assert_eq!(true, dir.path().exists());

    let meta = std::fs::metadata(dir.path());
    create(&dir);
    assert_eq!(meta.unwrap().modified().unwrap(), std::fs::metadata(dir.path()).unwrap().modified().unwrap());
}

// link_to -----------------------------------------------------------------------------------------

#[test_args(
    safe: |link:&Dir,to| { link.link_to_result(to, false).unwrap(); };
    easy: |link:&Dir,to| { link.link_to(to); };
    bild: |link:&Dir,to| { link.clone().linked_to(to); }
)]
fn dir_link_to(link_to:_) {
    let (_, root, dir) = dir_fix();
    dir.create();

    let link = root.dir("nested/link");
    link_to(&link, &dir);
    link_to(&link, &dir);

    assert_eq!(dir.path(), std::fs::read_link(link.path()).unwrap());
}

#[test_args(
    forc: |link:&Dir,to| { link.link_to_result(to, true).expect("Link error"); };
    safe: |link:&Dir,to| { link.link_to_result(to, false).expect("Link error"); } ! "Link error";
    easy: |link:&Dir,to| { link.link_to(to); }           ! "Creating link";
    bild: |link:&Dir,to| { link.clone().linked_to(to); } ! "Creating link"
)]
fn dir_link_to_overwrite_link(link_to:_) {
    let (_, root, dir) = dir_fix();
    dir.create();
    let foe = root.dir("foe");

    let link = root.dir("nested/link");
    link_to(&link, &dir);
    link_to(&link, &foe);
}

#[test_args(
    forc: |link:&Dir,to| { link.link_to_result(to, true).expect("Link error"); } ! "Link error";
    safe: |link:&Dir,to| { link.link_to_result(to, false).expect("Link error"); } ! "Link error";
    easy: |link:&Dir,to| { link.link_to(to); }           ! "Creating link";
    bild: |link:&Dir,to| { link.clone().linked_to(to); } ! "Creating link"
)]
fn dir_link_to_overwrite_dir(link_to:_) {
    let (_, root, dir) = dir_fix();
    dir.create();

    let link = root.dir("nested/link");
    link.create();
    link_to(&link, &dir);
}

// link_from_inside --------------------------------------------------------------------------------

#[test_args(
    safe: |dir:&Dir,from| { dir.link_from_inside_result(from, false).unwrap(); };
    easy: |dir:&Dir,from| { dir.link_from_inside(from); };
    bild: |dir:&Dir,from| { dir.clone().linked_from_inside(from); };
)]
fn dir_link_from_inside(link_from_inside:_) {
    let (_, root, dir) = dir_fix();
    let target = root.dir("target").created();

    link_from_inside(&target, &dir);
    link_from_inside(&target, &dir);

    assert_eq!(target.path(), std::fs::read_link(dir.dir("target").path()).unwrap());
}

#[test_args(
    forc: |dir:&Dir,from| { dir.link_from_inside_result(from, true).expect("Link err"); } ! "Link";
    safe: |dir:&Dir,from| { dir.link_from_inside_result(from, false).expect("Link err"); } ! "Link";
    easy: |dir:&Dir,from| { dir.link_from_inside(from); } ! "Creating link";
    bild: |dir:&Dir,from| { dir.clone().linked_from_inside(from) } ! "Creating link";
)]
fn dir_link_from_inside_overwrite_dir(link_from_inside:_) {
    let (_, root, dir) = dir_fix();
    let target = root.dir("target").created();
    dir.dir("target").create();

    link_from_inside(&target, &dir);
}

#[test_args(
    forc: |dir:&Dir,from| { dir.link_from_inside_result(from, true).expect("Link err"); };
    safe: |dir:&Dir,from| { dir.link_from_inside_result(from, false).expect("Link err"); } ! "Link";
    easy: |dir:&Dir,from| { dir.link_from_inside(from); } ! "Creating link";
    bild: |dir:&Dir,from| { dir.clone().linked_from_inside(from) } ! "Creating link";
)]
fn dir_link_from_inside_overwrite_link(link_from_inside:_) {
    let (_, root, dir) = dir_fix();
    let other = root.dir("foo").created();
    let target = root.dir("target").created();
    dir.dir("target").link_to(&other);

    link_from_inside(&target, &dir);
}

// timestamp ---------------------------------------------------------------------------------------

#[test]
fn dir_timestamp() {
    let (_, _, dir) = dir_fix();
    assert_eq!(None, dir.timestamp());

    dir.create();
    assert_eq!(std::fs::metadata(dir.path()).unwrap().modified().unwrap(), dir.timestamp().unwrap());
}

// touch -------------------------------------------------------------------------------------------

#[test_args(
    safe: |dir:&Dir| { dir.touch_result().unwrap(); };
    easy: |dir:&Dir| { dir.touch(); };
    bild: |dir:&Dir| { dir.clone().touched(); }
)]
fn dir_touch(touch:_) {
    let (_, _, dir) = dir_fix();

    //touch-create
    touch(&dir);
    assert!(dir.path().exists());

    //touch-modify
    let file = dir.file("dummy").created();
    let before = std::fs::metadata(dir.path()).unwrap().modified().unwrap();

    touch(&dir);
    assert!(file.path().exists());
    assert_eq!(true, before < std::fs::metadata(dir.path()).unwrap().modified().unwrap());
}

// ops ---------------------------------------------------------------------------------------------

#[test]
fn dir_add_dir() {
    let foo = Dir::from("/foo");
    let bar = Dir::from("/bar");
    let baz = Dir::from("/baz");

    let mut foobar = (&foo + &bar + &baz).into_iter();
    assert_eq!(Some(foo), foobar.next());
    assert_eq!(Some(bar), foobar.next());
    assert_eq!(Some(baz), foobar.next());
    assert_eq!(None, foobar.next());
}

#[test]
fn dir_add_file() {
    let foo = Dir::from("/foo");
    let baz = foo.file("baz");
    let mut foobaz = (&foo + &baz).into_iter();
    assert_eq!(Some(Unit::Dir(foo)), foobaz.next());
    assert_eq!(Some(Unit::File(baz)), foobaz.next());
    assert_eq!(None, foobaz.next());
}

// glob --------------------------------------------------------------------------------------------

#[test_args(
    none: "**/*.jpg", 0, 0;
    level1: "*", 1, 1;
    level2: "*/*", 2, 0;
    level3: "*/*/*", 2, 4;
    bar1: "**/bar1/*", 1, 2;
    rs: "**/*.rs", 0, 5;
    all: "**", 5, 9
)]
fn dir_content_count(glob: &str, dirs: usize, files: usize) {
    let temp = tempfile::tempdir().unwrap();
    let root = Dir::from(temp.path());

    root.dir("foo").create();
    root.file("root.rs").create();

    root.dir( "foo/bar1").create();
    root.file("foo/bar1/bar.rs").create();
    root.file("foo/bar1/bars").create();
    root.dir( "foo/bar1/baz").create();
    root.file("foo/bar1/baz/baz.rs").create();
    root.file("foo/bar1/baz/baz.js").create();
    root.dir( "foo/bar2").link_to(&root.dir("foo/bar1"));
    let cycle = root.dir( "foo/cycl").linked_to(&root);

    assert_eq!(dirs, root.dirs(glob).into_iter().count());
    assert_eq!(files, root.files(glob).into_iter().count());
    assert_eq!(dirs+files, root.content(glob).into_iter().count());
    assert_eq!(dirs+files, cycle.content(glob).into_iter().count());
}

#[test]
fn dir_content_incl_excl() {
    let temp = tempfile::tempdir().unwrap();
    let root = Dir::from(temp.path());

    root.file("foo.rs").create();
    root.file("bar.js").create();
    root.file("baz.js").create();

    assert_eq!(1, root.content("foo*").into_iter().count());
    assert_eq!(2, root.content("foo*").include("bar*").into_iter().count());
    assert_eq!(3, root.content("foo*").include("bar*").include("ba*").into_iter().count());
    assert_eq!(2, root.content("**").exclude("*.rs").into_iter().count());
    assert_eq!(1, root.content("**").exclude("*.rs").exclude("baz*").into_iter().count());
}

#[test]
fn dir_content_paths() {
    let temp = tempfile::tempdir().unwrap();
    let root = Dir::from(temp.path());

    let dir = root.dir("foo").created();
    let file = dir.file("bar");
    file.create();

    assert_eq!(dir, root.dirs("**").into_iter().next().unwrap());
    assert_eq!(file, root.files("**").into_iter().next().unwrap());
    assert_eq!(Unit::File(file), root.content("**/bar").into_iter().next().unwrap());
}

#[test]
fn dir_content_timestamp() {
    let temp = tempfile::tempdir().unwrap();
    let root = Dir::from(temp.path());

    let dir = root.dir("foo");
    let file = dir.file("bar");

    assert_eq!(None, dir.dirs("**").timestamp());
    assert_eq!(None, dir.files("**").timestamp());
    assert_eq!(None, dir.content("**").timestamp());

    dir.create();
    dir.file("bar").create();

    assert_eq!(dir.timestamp(), root.dirs("**").timestamp());
    assert_eq!(file.timestamp(), root.files("**").timestamp());
    assert_eq!(dir.timestamp(), root.content("**").timestamp());
}
