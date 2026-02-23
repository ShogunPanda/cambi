mod common;

use std::{fs, thread, time::Duration};

use assert_cmd::Command;
use serial_test::serial;
use tiny_http::{Method, Response, Server};

use crate::common::{commit_with_date, create_repo, git};

#[test]
#[serial]
fn release_skips_update_when_existing_payload_matches() {
  let repo = create_repo();
  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"x\"); }\n").expect("write feat file");
  commit_with_date(repo.path(), "feat: add output", "2026-02-22T10:00:00Z");
  git(repo.path(), &["tag", "v0.2.0"]);

  let server = Server::http("127.0.0.1:0").expect("start server");
  let base = format!("http://{}", server.server_addr());

  let body = "- feat: add output";
  let list = format!(
    "[{{\"id\":1,\"tag_name\":\"v0.2.0\",\"name\":\"0.2.0\",\"body\":\"{}\"}}]",
    body.replace('"', "\\\"")
  );

  thread::spawn(move || {
    let req = server
      .recv_timeout(Duration::from_secs(10))
      .expect("recv")
      .expect("some");
    assert_eq!(req.method(), &Method::Get);
    assert_eq!(req.url(), "/repos/o/r/releases?per_page=100");
    req
      .respond(Response::from_string(list).with_status_code(200))
      .expect("respond");
  });

  // SAFETY: serialized test restores env value.
  unsafe { std::env::set_var("CAMBI_GITHUB_API_BASE", base) };

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd
    .current_dir(repo.path())
    .args(["release", "--owner", "o", "--repo", "r", "--token", "t"]);
  cmd.assert().success();

  // SAFETY: serialized test restores env value.
  unsafe { std::env::remove_var("CAMBI_GITHUB_API_BASE") };
}
