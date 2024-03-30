use std::process::{ChildStdout, Command, Stdio};

pub fn tail(filepath: &str) -> Result<ChildStdout, String> {
    let child = Command::new("tail")
        .args(&["-f", "-n", "100", filepath])
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|err| err.to_string())?;

    Ok(child.stdout.unwrap())
}
