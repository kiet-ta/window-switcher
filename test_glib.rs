use gtk4::glib;
fn main() {
    let (s, r) = glib::MainContext::channel::<i32>(glib::Priority::DEFAULT);
}
