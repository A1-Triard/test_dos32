#![deny(warnings)]

use dos_cp::CodePage;
use dos_cp_generator::{KNOWN_CODE_PAGES, CodePageGenExt};
use std::fs::create_dir_all;

use std::env::var_os;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir = var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir).join("CODEPAGE");
    create_dir_all(&out_dir).unwrap();
    for &code_page in KNOWN_CODE_PAGES {
        let file = out_dir.join(format!("{}", code_page));
        let mut file = File::create(file).unwrap();
        file.write_all(&CodePage::generate(code_page).into_bytes()).unwrap();
    }
}
