use std::env;
use std::env::temp_dir;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use cairo_lang_starknet_classes::contract_class::ContractClass;

use crate::errors::CompilationUtilError;

// Solve Code duplication.
pub fn get_absolute_path(relative_path: &str) -> PathBuf {
    Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("../..").join(relative_path)
}

#[cfg(test)]
#[path = "compile_test.rs"]
pub mod compile_test;

const STARKNET_SIERRA_COMPILE_EXE: &str =
    "crates/starknet_sierra_compile/tmp/cargo/bin/starknet-sierra-compile";

struct SierraToCasmCompilationArgs {
    add_pythonic_hints: bool,
    max_bytecode_size: usize,
}

// TODO(Arni, 1/05/2024): Add the configurable parameters to the function.
pub fn compile_sierra_to_casm(
    contract_class: ContractClass,
) -> Result<CasmContractClass, CompilationUtilError> {
    env::set_current_dir(get_absolute_path("")).expect("Failed to set current dir.");

    let serialized_contract_class = serde_json::to_string(&contract_class).expect("number 1");

    // Create a temporary file path
    let mut temp_path = temp_dir();
    temp_path.push("temp_file.sierra.json");

    // Create and open the file
    let mut file = File::create(&temp_path).expect("number 2");

    // Write the content to the file
    file.write_all(serialized_contract_class.as_bytes()).expect("number 3");

    let compilation_args =
        SierraToCasmCompilationArgs { add_pythonic_hints: true, max_bytecode_size: 180000 };
    let compiler_path = STARKNET_SIERRA_COMPILE_EXE;

    let mut command = Command::new(compiler_path);
    command.arg(temp_path.to_str().expect("number 4"));

    // Add aditional arguments.
    if compilation_args.add_pythonic_hints {
        command.arg("--add-pythonic-hints");
    }
    // TODO(Arni): use max-bytecode-size.
    let _max_bytecode_size = compilation_args.max_bytecode_size;

    let compile_output =
        command.output().unwrap_or_else(|e| panic!("Failed to execute command: {}", e));

    if !compile_output.status.success() {
        let stderr_output = String::from_utf8(compile_output.stderr).expect("number 5"); // TODO: handle error
        return Err(CompilationUtilError::CompilationError(stderr_output));
    };

    Ok(serde_json::from_slice::<CasmContractClass>(&compile_output.stdout).expect("number 6"))
}
