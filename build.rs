#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/windows/icon.ico");
    res.compile().unwrap();
}

#[cfg(unix)]
fn main() {}
