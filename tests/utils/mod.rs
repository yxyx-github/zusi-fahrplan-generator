use std::process::Output;

#[allow(dead_code)]
pub fn print_output(output: Output) {
    println!("Output (stdout, stderr):");
    println!("{}", String::from_utf8(output.stdout).unwrap());
    eprintln!("{}", String::from_utf8(output.stderr).unwrap());
}