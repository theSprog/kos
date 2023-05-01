use std::fs::{read_dir, File};
use std::io::{Result, Write};

static SRC_PATH: &str = "./user/src/";
static TARGET_PATH: &str = "./user/prog/";
static BIN_PATH: &str = "./user/bin/";
static SCRIPT_PATH: &str = "./src/link_app.S";

fn main() {
    println!("cargo:rerun-if-changed={}", SRC_PATH);
    println!("cargo:rerun-if-changed={}", TARGET_PATH);
    insert_app_data().expect("insert app data failed");
}

fn insert_app_data() -> Result<()> {
    // 运行时动态创建 link_app.S
    let mut f = File::create(SCRIPT_PATH).unwrap();
    let mut apps: Vec<_> = read_dir(BIN_PATH)
        .unwrap()
        .into_iter()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect();

    // 按照首字母排序
    apps.sort();

    // 向 link_app.S 中写入内容
    writeln!(
        f,
        r#"
    .align 3
    .section .data
    .global _num_app
_num_app:
    .quad {}"#,
        apps.len()
    )?;

    for i in 0..apps.len() {
        writeln!(f, r#"    .quad app_{}_start"#, i)?;
    }
    writeln!(f, r#"    .quad app_{}_end"#, apps.len() - 1)?;

    for (idx, app) in apps.iter().enumerate() {
        writeln!(
            f,
            r#"
    .section .data
    .global app_{0}_start
    .global app_{0}_end
app_{0}_start:
    .incbin "{2}{1}.bin"
app_{0}_end:"#,
            idx, app, BIN_PATH
        )?;
    }
    Ok(())
}
