use dy::*;
use std::env;
use std::process::{Command, Stdio};

#[cfg(feature = "import")]
fn build_cargo(current_dir: &String, is_release: bool) {
    let mut command = Command::new("cargo");

    command.arg("build");
    if is_release {
        command.arg("--release");
    }
    command
        .current_dir(current_dir)
        .stdout(Stdio::inherit())
        .output()
        .unwrap();
}

#[test]
#[cfg(feature = "import")]
fn import_dll_test() {
    let profile = env!("PROFILE");
    let is_release = profile == "release";
    let crate_path = format!("{}/tests/import_test", env!("CARGO_MANIFEST_DIR"));
    build_cargo(&crate_path, is_release);

    let m = Module::new(
        "import_test",
        &[&format!("{}/target/{}", crate_path, profile)],
    )
    .unwrap();
    let f = m.get_fn("multiply_two_only_numbers").unwrap();
    let args = vec![
        Value::new_int(5),
        Value::new_float(6.3),
        Value::new_str("Hello"),
    ];
    let res = f.call(args);
    let res = res.as_arr().unwrap();
    for elem in res.iter() {
        match elem.as_type() {
            As::Int(i) => assert_eq!(i.get(), 10),
            As::Float(f) => assert_eq!(f.get(), 12.6),
            As::Str(s) => assert_eq!(s.get(), "Hello"),
            _ => panic!("Invalid type"),
        }
    }
}
