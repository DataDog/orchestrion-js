/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2025 Datadog, Inc.
 **/
use assert_cmd::prelude::*;
use orchestrion_js::*;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;
use swc::config::IsModule;

static TEST_MODULE_NAME: &str = "undici";
static TEST_MODULE_PATH: &str = "index.mjs";

pub fn transpile_and_test(test_file: &str, mjs: bool, config: Config) {
    let test_file = PathBuf::from(test_file);
    let test_dir = test_file.parent().expect("Couldn't find test directory");

    let file_path = PathBuf::from("index.mjs");
    let instrumentor = Instrumentor::new(config);
    let mut instrumentations =
        instrumentor.get_matching_instrumentations(TEST_MODULE_NAME, "0.0.1", &file_path);

    let extension = if mjs { "mjs" } else { "js" };
    let instrumentable = test_dir.join(format!("mod.{}", extension));
    let mut file = std::fs::File::open(&instrumentable).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let result = instrumentations
        .transform(&contents, IsModule::Bool(mjs))
        .unwrap();

    let instrumented_file = test_dir.join(format!("instrumented.{}", extension));
    let mut file = std::fs::File::create(&instrumented_file).unwrap();
    file.write_all(result.as_bytes()).unwrap();

    let test_file = format!("test.{}", extension);
    Command::new("node")
        .current_dir(test_dir)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .arg(&test_file)
        .assert()
        .success();
}

pub fn test_module_matcher() -> ModuleMatcher {
    ModuleMatcher::new(TEST_MODULE_NAME, ">=0.0.1", TEST_MODULE_PATH).unwrap()
}
