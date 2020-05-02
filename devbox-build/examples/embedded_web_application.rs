use devbox_build::{Build, Cmd, Resource};

pub fn main() {

    let build = Build::new();

    //-- Setup web app build dir inside of Rust target directory -----------------------------------

    // Rust does not allow changes outside target directory, so setup a webapp build directoy
    // using links to source files where nodejs and company can do it's thing

    let webrs = build.out_dir().file("webapp.rs");
    let websrc = build.manifest_dir().dir("webapp");
    let webwrk = build.out_dir().dir("webapp_build");
    let webwrk_pkg = webwrk.file("package.json");
    let webwrk_pkl = webwrk.file("package-lock.json");
    let webwrk_ndm = webwrk.dir("node_modules");
    let webwrk_dst = webwrk.dir("dist");

    for unit in websrc.content("*") {
        unit.link_from_inside(&webwrk);
    }

    //-- Build webapp using NPM --------------------------------------------------------------------

    let npm = Cmd::new("npm").arg("--prefix").arg(webwrk.path());

    webwrk_ndm.mk_from("Install WebApp node packages", &webwrk_pkg + &webwrk_pkl, ||{
        npm.clone().arg("install").run();
        webwrk_ndm.touch();
    });

    webwrk_dst.mk_from("Build WebApp using webpack", &webwrk.content("**"), || {
        npm.clone().arg("run").arg("build").run();
        webwrk_dst.touch();
    });

    //-- Package webapp into server binary as Rust source code -------------------------------------

    webrs.mk_from("Embed WebApp build into binary", &webwrk_dst, || {
        let mappings = webwrk_dst.files("**").into_iter().map(|file|
            format!(r#""{}" => Some(include_bytes!("{}")),"#,
                file.path().strip_prefix(&webwrk_dst.path()).unwrap().to_str().unwrap(),
                file.path().to_str().unwrap())
        ).fold("".to_owned(), |result, ref s| result + s + "\n" );

        webrs.rewrite(format!(r"
            pub fn file(path: &str) -> Option<&'static [u8]> {{
                match path {{
                    {}
                    &_ => None,
                }}
            }}
        ", mappings));
    });
}