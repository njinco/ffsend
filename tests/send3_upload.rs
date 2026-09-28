#![cfg(feature = "send3")]

use std::fs;
use std::net::TcpListener;
use std::process::Command;
use std::thread;

use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response},
    http::HeaderValue,
    Message,
};

fn upload_to_test_server(
    contents: &[u8],
    accepted: bool,
    basic_auth: bool,
) -> (std::process::Output, usize) {
    let server = TcpListener::bind("127.0.0.1:0").expect("bind test WebSocket server");
    let address = server.local_addr().expect("test server address");
    let host = format!("http://{}/", address);
    let response_url = format!("http://{}/download/testfile123", address);
    let server_thread = thread::spawn(move || {
        let (stream, _) = server.accept().expect("receive WebSocket upgrade");
        let mut socket = accept_hdr(stream, |request: &Request, mut response: Response| {
            assert_eq!(request.uri().path(), "/api/ws");
            assert_eq!(request.headers()["Sec-WebSocket-Protocol"], "ffsend");
            if basic_auth {
                assert_eq!(request.headers()["Authorization"], "Basic dXNlcjpwYXNz");
            } else {
                assert!(!request.headers().contains_key("Authorization"));
            }
            response
                .headers_mut()
                .insert("Sec-WebSocket-Protocol", HeaderValue::from_static("ffsend"));
            Ok(response)
        })
        .expect("upgrade WebSocket");

        let metadata = match socket.read().expect("receive upload metadata") {
            Message::Text(text) => text,
            other => panic!("expected metadata text, got {:?}", other),
        };
        assert!(metadata.contains("\"fileMetadata\""));
        assert!(metadata.contains("\"authorization\""));
        assert!(
            !metadata.contains("\"dlimit\""),
            "the default download limit must be omitted so the server applies its default"
        );
        assert!(!metadata.contains("distinctive-test-plaintext"));
        socket
            .send(Message::Text(
                format!(
                    "{{\"id\":\"testfile123\",\"url\":\"{}\",\"ownerToken\":\"test-owner\"}}",
                    response_url
                )
                .into(),
            ))
            .expect("send upload initialization");

        let header = socket.read().expect("receive encrypted header");
        assert!(matches!(header, Message::Binary(ref data) if !data.is_empty()));

        let mut chunks = 0;
        loop {
            match socket.read().expect("receive encrypted upload") {
                Message::Binary(data) if data.as_ref() == [0] => break,
                Message::Binary(data) => {
                    assert!(!data.is_empty());
                    assert!(!data
                        .windows(b"distinctive-test-plaintext".len())
                        .any(|window| window == b"distinctive-test-plaintext"));
                    chunks += 1;
                }
                other => panic!("expected binary upload frame, got {:?}", other),
            }
        }
        assert!(chunks > 0);
        socket
            .send(Message::Text(format!("{{\"ok\":{}}}", accepted).into()))
            .expect("send upload status");
        chunks
    });

    let directory = tempfile::tempdir().expect("create test directory");
    let source = directory.path().join("upload.txt");
    fs::write(&source, contents).expect("write upload data");
    let mut command = Command::new(env!("CARGO_BIN_EXE_ffsend"));
    command
        .args([
            "--api",
            "3",
            "--no-interact",
            "--quiet",
            "upload",
            "--host",
            &host,
        ])
        .arg(&source)
        .env("FFSEND_HISTORY", directory.path().join("history.json"))
        .env_remove("FFSEND_ARCHIVE")
        .env_remove("FFSEND_SHORTEN")
        .env_remove("FFSEND_OPEN")
        .env_remove("FFSEND_COPY")
        .env_remove("FFSEND_BASIC_AUTH");
    if basic_auth {
        command.args(["--basic-auth", "user:pass"]);
    }
    let output = command.output().expect("run ffsend upload");
    let chunks = server_thread.join().expect("test server thread");
    (output, chunks)
}

#[test]
fn send3_upload_sends_encrypted_chunks_and_returns_a_link() {
    let contents = "distinctive-test-plaintext".repeat(10_000);
    let (output, chunks) = upload_to_test_server(contents.as_bytes(), true, false);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        chunks > 1,
        "large upload should span multiple WebSocket frames"
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("/download/testfile123#"));
}

#[test]
fn send3_upload_accepts_a_small_file() {
    let (output, chunks) = upload_to_test_server(b"distinctive-test-plaintext", true, false);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(chunks, 1);
    assert!(String::from_utf8_lossy(&output.stdout).contains("/download/testfile123#"));
}

#[test]
fn send3_upload_rejects_a_failed_server_status() {
    let (output, _) = upload_to_test_server(b"distinctive-test-plaintext", false, false);
    assert!(!output.status.success());
    assert!(!String::from_utf8_lossy(&output.stdout).contains("/download/testfile123#"));
}

#[test]
fn send3_upload_preserves_proxy_basic_auth() {
    let (output, _) = upload_to_test_server(b"distinctive-test-plaintext", true, true);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
