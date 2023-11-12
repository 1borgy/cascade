#[cfg(windows)]
fn main() {
    println!("hie!!!");
    let mut res = winres::WindowsResource::new();
    res.set_icon("../resources/cascade.ico");
    res.compile().unwrap();
}
