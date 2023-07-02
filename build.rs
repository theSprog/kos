use std::fs::{read_dir, File};
use std::io::{Result, Write};
use std::println;

static SRC_PATH: &str = "user/src/";
static TARGET_PATH: &str = "user/prog/";
static SCRIPT_PATH: &str = "kernel/link_app.S";

fn main() {
    println!("cargo:rerun-if-changed={}", SRC_PATH);
    println!("cargo:rerun-if-changed={}", TARGET_PATH);
    insert_app_data().expect("insert app data failed");
}

fn insert_app_data() -> Result<()> {
    // 运行时动态创建 link_app.S
    let mut f = File::create(SCRIPT_PATH).unwrap();
    let mut apps: Vec<_> = read_dir(TARGET_PATH)
        .unwrap()
        .map(|dir_entry| dir_entry.unwrap().file_name().into_string().unwrap())
        .collect();

    // 按照首字母排序
    apps.sort();

    // 向 link_app.S 中写入内容
    writeln!(
        f,
        r#".align 3    # 按照 2^3 = 8字节 对齐
    .section .data
    .global _num_app
_num_app:
    .quad {}"#,
        apps.len()
    )?;

    for i in 0..apps.len() {
        writeln!(f, r#"    .quad app_{}_start"#, i)?;
    }

    if !apps.is_empty() {
        writeln!(f, r#"    .quad app_{}_end"#, apps.len() - 1)?;

        writeln!(
            f,
            r#"
    .global _app_names
_app_names:"#
        )?;
        for app in apps.iter() {
            writeln!(f, r#"    .string "{}{}""#, TARGET_PATH, app)?;
        }

        for (idx, app) in apps.iter().enumerate() {
            writeln!(
                f,
                r#"
    .section .data
    .global app_{0}_start
    .global app_{0}_end
    .align 3
app_{0}_start:
    .incbin "{2}{1}"
app_{0}_end:"#,
                idx, app, TARGET_PATH
            )?;
        }
    }

    Ok(())
}
