fn main() {
    slint_build::compile("ui/appwindow.slint").unwrap();
    #[cfg(target_os = "windows")]
    {
        // Add your lib from Peak CAN here. please refer:
        //   https://github.com/TuEmb/peak-can-rs/blob/main/README.md

        // println!("cargo:rustc-link-search=native=<path to your lib>");
        // println!("cargo:rustc-link-lib=static=<your lib name>");
    }
}
