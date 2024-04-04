use std::process::Command;

fn main() {
    install_starknet_sierra_compile();
}

fn install_starknet_sierra_compile() {
    println!("Installing starknet-sierra-compile");
    let mut command = Command::new("cargo");
    command.arg("install");
    command.arg("--root");
    command.arg("tmp/cargo"); // TODO: DOn't dup the path.
    command.arg("starknet-sierra-compile");
    let result = command.output();
    if let Err(e) = result {
        panic!("Failed to execute command: {}", e);
    }
    let compile_output = result.unwrap(); // TODO: handle error
    let stderr_output = String::from_utf8(compile_output.stderr).unwrap(); // TODO: handle error
    if !compile_output.status.success() {
        panic!("Failed to compile Sierra code: {}", stderr_output);
    };
}
