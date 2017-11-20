use std::path::Path;

#[cfg(not(target_os = "android"))]
fn check_assets_dir() {
    use std::fs;
    use std::process;

    // TODO: check assets version
    if let Err(e) = fs::metadata("assets") {
        println!("Can`t find 'assets' dir: {}", e);
        println!("Note: see 'Assets' section of README.rst");
        process::exit(1);
    }
}

pub fn load_as_string<P: AsRef<Path>>(path: P) -> String {
    String::from_utf8(load(path)).unwrap()
}

#[cfg(not(target_os = "android"))]
pub fn load<P: AsRef<Path>>(path: P) -> Vec<u8> {
    use std::fs::File;
    use std::io::Read;

    check_assets_dir();
    let mut buf = Vec::new();
    let fullpath = &Path::new("assets").join(&path);
    let mut file = match File::open(&fullpath) {
        Ok(file) => file,
        Err(err) => {
            panic!("Can`t open file '{}' ({})", fullpath.display(), err)
        }
    };
    match file.read_to_end(&mut buf) {
        Ok(_) => buf,
        Err(err) => {
            panic!("Can`t read file '{}' ({})", fullpath.display(), err)
        }
    }
}

#[cfg(target_os = "android")]
pub fn load<P: AsRef<Path>>(path: P) -> Vec<u8> {
    use android_glue;

    // TODO: check assets version
    let filename = path.as_ref().to_str().expect("Can`t convert Path to &str");
    match android_glue::load_asset(filename) {
        Ok(buf) => buf,
        Err(_) => panic!("Can`t load asset '{}'", filename),
    }
}
