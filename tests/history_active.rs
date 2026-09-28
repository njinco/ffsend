#![cfg(feature = "history")]

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::thread;

use ffsend_api::file::remote_file::RemoteFile;
use ffsend_api::url::Url;

fn saved_file(host: &str, id: &str) -> RemoteFile {
    let host: Url = host.parse().expect("parse test host");
    let url = host
        .join(&format!("download/{}", id))
        .expect("build test share URL");
    RemoteFile::new_now(id.into(), host, url, vec![1; 16], None)
}

fn run_with_responses(responses: &[(&str, &str)]) -> (std::process::Output, String) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let host = format!("http://{}/", listener.local_addr().unwrap());
    let ids: Vec<&str> = responses.iter().map(|(id, _)| *id).collect();
    let files: Vec<RemoteFile> = ids.iter().map(|id| saved_file(&host, id)).collect();
    let history_data = format!(
        "version = \"0.2.77\"\n{}",
        files
            .iter()
            .map(|file| format!("[[files]]\n{}", toml::to_string(file).unwrap()))
            .collect::<String>()
    );

    let directory = tempfile::tempdir().unwrap();
    let history_path = directory.path().join("history.toml");
    fs::write(&history_path, &history_data).unwrap();

    let replies: Vec<(String, String)> = responses
        .iter()
        .map(|(id, reply)| ((*id).into(), (*reply).into()))
        .collect();
    let server = thread::spawn(move || {
        for _ in 0..replies.len() {
            let (mut stream, _) = listener.accept().expect("receive availability check");
            let mut request = [0; 2048];
            let count = stream.read(&mut request).expect("read request");
            let request = String::from_utf8_lossy(&request[..count]);
            let (_, reply) = replies
                .iter()
                .find(|(id, _)| request.starts_with(&format!("GET /api/exists/{} ", id)))
                .expect("unexpected availability request");
            stream.write_all(reply.as_bytes()).expect("send response");
        }
    });

    let output = Command::new(env!("CARGO_BIN_EXE_ffsend"))
        .args(["--quiet", "--history"])
        .arg(&history_path)
        .args(["history", "--active"])
        .output()
        .expect("run history --active");
    server.join().expect("test server");
    assert_eq!(fs::read_to_string(history_path).unwrap(), history_data);
    (output, host)
}

#[test]
fn active_history_shows_only_available_links() {
    let available = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 26\r\n\r\n{\"requiresPassword\":false}";
    let missing = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
    let (output, _) = run_with_responses(&[("active123", available), ("missing123", missing)]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let links = String::from_utf8_lossy(&output.stdout);
    assert!(links.contains("/download/active123#"));
    assert!(!links.contains("/download/missing123#"));
}

#[test]
fn active_history_does_not_hide_a_server_error() {
    let failed = "HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\n\r\n";
    let (output, _) = run_with_responses(&[("failed123", failed)]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
}
