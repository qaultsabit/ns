use ssh2::Session;
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;

const ADDRESS: &str = "localhost:2222";
const USER: &str = "tsabit";
const PASSWORD: &str = "password";
const LOG_DIR: &str = "/home/tsabit/logs";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: ns <srn> <dest>");
        return;
    }

    let srn = &args[1];
    let dest = &args[2];

    create_dir_all(dest).expect("Failed to create destination directory");

    let tcp = TcpStream::connect(ADDRESS).expect("Failed to connect to SSH server");
    let mut sess = Session::new().expect("Failed to create SSH session");
    sess.set_tcp_stream(tcp);
    sess.handshake().expect("SSH handshake failed");
    sess.userauth_password(USER, PASSWORD)
        .expect("Authentication failed");

    let mut channel = sess
        .channel_session()
        .expect("Failed to create SSH channel");
    let cmd = format!(
        "grep -rl {} --include='history*' --include='message*' {} | xargs -I {{}} basename {{}}",
        srn, LOG_DIR
    );
    channel.exec(&cmd).expect("Failed to execute grep command");

    let mut result = String::new();
    channel
        .read_to_string(&mut result)
        .expect("Failed to read command output");

    let mut log_files: Vec<String> = result.lines().map(|s| s.trim().to_string()).collect();
    log_files.push(String::from("ndclog2.log"));

    if log_files.is_empty() {
        eprintln!("No matching log files found.");
        return;
    }

    for log_file in log_files {
        let remote_path = format!("{}/{}", LOG_DIR, log_file);
        let local_path = format!("{}/{}", dest, log_file);

        match sess.scp_recv(Path::new(&remote_path)) {
            Ok((mut remote_file, _)) => {
                let mut local_file =
                    File::create(&local_path).expect("Failed to create local file");
                let mut buffer = Vec::new();
                remote_file
                    .read_to_end(&mut buffer)
                    .expect("Failed to read remote file");
                local_file
                    .write_all(&buffer)
                    .expect("Failed to write to local file");
                println!("{log_file}");
            }
            Err(err) => eprintln!("Failed to download {}: {}", remote_path, err),
        }
    }
}
