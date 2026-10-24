use std::process::Command;

pub fn shell(unix: &str, windows: &str) -> Command {
    if cfg!(windows) {
        let mut command = Command::new("cmd");
        command.args(["/C", windows]);
        command
    } else {
        let mut command = Command::new("sh");
        command.args(["-c", unix]);
        command
    }
}

pub fn sleeping(seconds: u32) -> Command {
    shell(
        &format!("sleep {seconds}"),
        &format!("ping -n {} 127.0.0.1 >nul", seconds + 1),
    )
}
