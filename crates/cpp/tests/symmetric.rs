use std::{
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
    process::Command,
};

use wit_bindgen_core::wit_parser::{Resolve, WorldId};

fn tester_source_file(dir_name: &str, tester_source_dir: &PathBuf) -> Option<PathBuf> {
    let mut tester_source_file = tester_source_dir.clone();
    tester_source_file.push(&format!("{dir_name}.rs"));
    if matches!(std::fs::exists(&tester_source_file), Ok(true)) {
        Some(tester_source_file)
    } else {
        None
    }
}

fn create_cargo_files(
    dir_name: &str,
    out_dir: &PathBuf,
    toplevel: &PathBuf,
    source_files: &PathBuf,
    tester_source_dir: &PathBuf,
) -> io::Result<()> {
    let Some(tester_source_file) = tester_source_file(dir_name, tester_source_dir) else {
        println!("Skipping {}", dir_name);
        return Ok(());
    };

    let mut out_dir = out_dir.clone();
    out_dir.push(dir_name);
    // println!("{cpp:?} {out_dir:?}");

    let mut dir = source_files.clone();
    dir.push(dir_name);

    drop(std::fs::remove_dir_all(&out_dir));
    std::fs::create_dir_all(&out_dir)?;
    let mut testee_dir = out_dir.clone();
    testee_dir.push("rust");
    std::fs::create_dir(&testee_dir)?;
    let testee_cargo = format!(
        "[package]\n\
            name = \"{dir_name}\"\n\
            publish = false\n\
            edition = \"2021\"\n\
            \n\
            [dependencies]\n\
            wit-bindgen = {{ path = \"{toplevel}/crates/guest-rust\" }}\n\
            test-rust-wasm = {{ path = \"{toplevel}/crates/cpp/tests/symmetric_tests/test-rust-wasm\" }}\n\
            futures = \"0.3\"\n\
            once_cell = \"1.20\"\n\
            \n\
            [lib]\n\
            crate-type = [\"cdylib\"]\n\
            ",
        toplevel = toplevel.display()
    );
    let mut filename = testee_dir.clone();
    filename.push("Cargo.toml");
    File::create(&filename)?.write_all(testee_cargo.as_bytes())?;
    drop(testee_cargo);
    // let mut testee_dir = out_dir.clone();
    // testee_dir.push("rust");
    //let mut filename = testee_dir.clone();
    filename.pop();
    filename.push("src");
    std::fs::create_dir(&filename)?;
    filename.push(format!("lib.rs"));
    let mut original = dir.clone();
    original.push("wasm.rs");
    std::os::unix::fs::symlink(original, filename)?;

    let tester_cargo = format!(
        "[package]\n\
            name = \"tester-{dir_name}\"\n\
            publish = false\n\
            edition = \"2021\"\n\
            \n\
            [dependencies]\n\
            wit-bindgen = {{ path = \"{toplevel}/crates/guest-rust\" }}\n\
            {dir_name} = {{ path = \"rust\" }}\n\
            futures = \"0.3\"\n\
            once_cell = \"1.20\"\n\
            ",
        toplevel = toplevel.display()
    );
    let mut filename = out_dir.clone();
    filename.push("Cargo.toml");
    File::create(&filename)?.write_all(tester_cargo.as_bytes())?;
    filename.pop();
    // let mut filename = out_dir.clone();
    filename.push("src");
    std::fs::create_dir(&filename)?;
    filename.push(format!("main.rs"));
    std::os::unix::fs::symlink(tester_source_file, &filename)?;

    Ok(())
}

fn tests(
    dir_name: &str,
    out_dir: &PathBuf,
    _toplevel: &PathBuf,
    source_files: &PathBuf,
    tester_source_dir: &PathBuf,
) -> io::Result<()> {
    // modelled after wit-bindgen/tests/runtime/main.rs
    let Some(_tester_source_file) = tester_source_file(dir_name, tester_source_dir) else {
        println!("Skipping {}", dir_name);
        return Ok(());
    };

    let mut dir = source_files.clone();
    dir.push(dir_name);

    // let mut rust = Vec::new();
    let mut cpp = Vec::new();
    for file in dir.read_dir()? {
        let path = file?.path();
        match path.extension().and_then(|s| s.to_str()) {
            // Some("rs") => rust.push(path),
            Some("cpp") => cpp.push(path),
            _ => {}
        }
    }

    let mut out_dir = out_dir.clone();
    out_dir.push(dir_name);
    // println!("{cpp:?} {out_dir:?}");

    let mut testee_dir = out_dir.clone();
    testee_dir.push("rust");
    let mut filename = testee_dir.clone();
    filename.push("src");
    //    std::fs::create_dir(&filename)?;
    filename.push(format!("lib.rs"));
    let mut original = dir.clone();
    original.push("wasm.rs");
    //    std::os::unix::fs::symlink(original, filename)?;

    let mut filename = out_dir.clone();
    filename.push("src");
    //    std::fs::create_dir(&filename)?;
    filename.push(format!("main.rs"));
    //    std::os::unix::fs::symlink(tester_source_file, &filename)?;

    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .current_dir(testee_dir)
        .env("RUSTFLAGS", "-Ltarget/debug")
        .env("SYMMETRIC_ABI", "1")
        .env("WIT_BINDGEN_DEBUG", "1");
    let status = cmd.status().unwrap();
    assert!(status.success());

    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .current_dir(&out_dir)
        .env("RUSTFLAGS", "-Ltarget/debug")
        .env("SYMMETRIC_ABI", "1")
        .env("WIT_BINDGEN_DEBUG", "1");
    let status = cmd.status().unwrap();
    assert!(status.success());

    for path in cpp.iter() {
        let (mut resolve, mut world) = resolve_wit_dir(&dir);
        let world_name = &resolve.worlds[world].name;
        let cpp_dir = out_dir.join("cpp");
        drop(fs::remove_dir_all(&cpp_dir));
        fs::create_dir_all(&cpp_dir).unwrap();

        let snake = world_name.replace("-", "_");
        let mut files = Default::default();
        let mut opts = wit_bindgen_cpp::Opts::default();
        opts.symmetric = true;
        if let Some(path) = path.file_name().and_then(|s| s.to_str()) {
            if path.contains(".new.") {
                opts.new_api = true;
            }
        }
        let mut cpp = opts.build();
        cpp.apply_resolve_options(&mut resolve, &mut world);
        cpp.generate(&resolve, world, &mut files).unwrap();

        for (file, contents) in files.iter() {
            let dst = cpp_dir.join(file);
            fs::write(dst, contents).unwrap();
        }

        let compiler = "clang++";
        let mut cmd = Command::new(compiler);
        let out_name = cpp_dir.join(format!("lib{}.so", dir_name));
        cmd.arg(path)
            .arg(cpp_dir.join(format!("{snake}.cpp")))
            .arg("-shared")
            .arg("-fPIC")
            .arg("-I")
            .arg(&cpp_dir)
            .arg("-I")
            .arg(&(String::from(env!("CARGO_MANIFEST_DIR")) + "/test_headers"))
            .arg("-Wall")
            .arg("-Wextra")
            .arg("-Wno-unused-parameter")
            .arg("-std=c++17")
            .arg("-g")
            .arg("-o")
            .arg(&out_name);
        println!("{:?}", cmd);
        let output = match cmd.output() {
            Ok(output) => output,
            Err(e) => panic!("failed to spawn compiler: {}", e),
        };

        if !output.status.success() {
            println!("status: {}", output.status);
            println!("stdout: ------------------------------------------");
            println!("{}", String::from_utf8_lossy(&output.stdout));
            println!("stderr: ------------------------------------------");
            println!("{}", String::from_utf8_lossy(&output.stderr));
            panic!("failed to compile");
        } else {
            let mut tester = out_dir.clone();
            tester.pop();
            tester.push("target");
            tester.push("debug");
            tester.push(&format!("tester-{dir_name}"));
            let run = Command::new(tester)
                .env("LD_LIBRARY_PATH", cpp_dir)
                .output();
            match run {
                Ok(output) => {
                    if !output.status.success() {
                        println!("status: {}", output.status);
                        println!("stdout: ------------------------------------------");
                        println!("{}", String::from_utf8_lossy(&output.stdout));
                        println!("stderr: ------------------------------------------");
                        println!("{}", String::from_utf8_lossy(&output.stderr));
                        panic!("failed to run");
                    }
                }
                Err(e) => panic!("failed to run tester: {}", e),
            }
        }
    }

    Ok(())
}

fn resolve_wit_dir(dir: &PathBuf) -> (Resolve, WorldId) {
    let mut resolve = Resolve::new();
    let (pkg, _files) = resolve.push_path(dir).unwrap();
    let world = resolve.select_world(pkg, None).unwrap();
    (resolve, world)
}

#[test]
fn symmetric_integration() -> io::Result<()> {
    let mut out_dir = std::env::current_exe()?;
    out_dir.pop();
    out_dir.pop();
    out_dir.pop();
    out_dir.push("symmetric-tests");
    if !out_dir.try_exists().unwrap_or(false) {
        std::fs::create_dir_all(&out_dir)?;
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let mut toplevel = manifest_dir.clone();
    toplevel.pop();
    toplevel.pop();

    let mut test_link = out_dir.clone();
    test_link.push("tests");
    if !test_link.try_exists().unwrap_or(false) {
        let mut original = toplevel.clone();
        original.push("tests");
        std::os::unix::fs::symlink(original, &test_link)?;
    }

    let mut source_files = toplevel.clone();
    source_files.push("tests");
    source_files.push("runtime");

    let mut tester_source_dir = manifest_dir.clone();
    tester_source_dir.push("tests");
    tester_source_dir.push("symmetric_tests");

    let default_testcases = vec![
        "flavorful",
        "lists",
        "many_arguments",
        "numbers",
        "options",
        "records",
        "results",
        "smoke",
        "strings",
    ];
    let testcases: Vec<String> = std::env::var_os("SYMMETRIC_TESTS").map_or_else(
        || {
            default_testcases
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        },
        |var| {
            var.into_string()
                .expect("UTF8 expected")
                .split(',')
                .map(|s| s.to_string())
                .collect()
        },
    );
    // create workspace
    {
        let mut workspace = format!(
            "[workspace]\n\
                resolver = \"2\"\n\
                \n\
                members = [\n"
        );
        for dir_name in testcases.iter() {
            if tester_source_file(dir_name, &tester_source_dir).is_some() {
                workspace.push_str(&format!(
                    "    \"{}\",\n    \"{}/rust\",\n",
                    dir_name, dir_name
                ));
            }
            create_cargo_files(
                dir_name,
                &out_dir,
                &toplevel,
                &source_files,
                &tester_source_dir,
            )?;
        }
        workspace.push_str("]\n");
        let mut filename = out_dir.clone();
        filename.push("Cargo.toml");
        File::create(&filename)?.write_all(workspace.as_bytes())?;
    }
    for dir_name in testcases {
        tests(
            &dir_name,
            &out_dir,
            &toplevel,
            &source_files,
            &tester_source_dir,
        )?;
    }

    Ok(())
}
