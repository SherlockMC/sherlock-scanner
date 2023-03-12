use std::process::Command;

fn main() {

    let output = Command::new("cmd")
        .args(&["/C", "runas", "/user:Administrator", "masscan", "-p1-65535", "142.250.177.46"])
        .output()
        .expect("failed to execute process");

    println!("status: {}", output.status);
    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    
}
