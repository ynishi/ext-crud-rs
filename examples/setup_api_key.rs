use log::{debug, error};
use std::process::Command;
use std::str;

fn main() {
    let output = Command::new("supabase")
        .arg("status")
        .output()
        .expect("failed to execute process");

    debug!("stdout: {}", str::from_utf8(&output.stdout).unwrap());

    // 標準出力の内容を文字列に変換
    let stdout = str::from_utf8(&output.stdout).expect("Invalid UTF-8");
    debug!("stdout: {}", stdout);

    // 各行をループして "service" を含む行を探す
    let service_line = stdout
        .lines()
        .find(|line| line.contains("service_role"))
        .expect("Service line not found");
    debug!("service_line: {}", service_line);

    // "service" キーと値のペアを分割
    if let Some((_, service_name)) = service_line.split_once(':') {
        // サービス名をトリムして出力
        let service_role_key = service_name.trim();
        println!("{}", service_role_key);
    } else {
        error!("Service name not found in the line.");
    }
}
