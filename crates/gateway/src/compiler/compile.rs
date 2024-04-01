use std::process::Command;

#[cfg(test)]
#[path = "compile_test.rs"]
pub mod compile_test;

const STARKNET_SIERRA_COMPILE_EXE: &str = "src/compiler/starknet-sierra-compile";

// TODO(Arni, 1/05/2024): Add the configurable parameters to the function.
pub fn compile_sierra_to_casm(sierra_path: &str) -> Vec<u8> {
    let add_pythonic_hints = true; // TODO: make this configurable?
    let compiler_path = STARKNET_SIERRA_COMPILE_EXE;

    let mut command = Command::new(compiler_path);
    command.arg(sierra_path);

    // Add aditional arguments.
    if add_pythonic_hints {
        command.arg("--add-pythonic-hints");
    }

    let result = command.output();
    if let Err(e) = result {
        panic!("Failed to execute command: {}", e);
    }
    let compile_output = result.unwrap(); // TODO: handle error
    let stderr_output = String::from_utf8(compile_output.stderr).unwrap(); // TODO: handle error
    if !compile_output.status.success() {
        panic!("Failed to compile Sierra code: {}", stderr_output);
    };

    compile_output.stdout
}
