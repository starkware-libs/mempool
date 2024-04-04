use std::process::Command;

#[cfg(test)]
#[path = "compile_test.rs"]
pub mod compile_test;

const STARKNET_SIERRA_COMPILE_EXE: &str = "tmp/cargo/bin/starknet-sierra-compile";

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

    let compile_output = command
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute command: {}", e));
    if !compile_output.status.success() {
        let stderr_output = String::from_utf8(compile_output.stderr).unwrap(); // TODO: handle error
        panic!("Failed to compile Sierra code: {}", stderr_output);
    };

    compile_output.stdout
}
