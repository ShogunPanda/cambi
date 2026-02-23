mod common;

use std::{
  fs,
  sync::{Arc, Mutex},
  thread,
  time::Duration,
};

use assert_cmd::Command;
use serial_test::serial;
use tiny_http::{Method, Response, Server};

use crate::common::{commit_with_date, create_repo, git};

fn spawn_mock_github(responses: Vec<(Method, String, u16, String)>) -> (String, Arc<Mutex<Vec<String>>>) {
  let server = Server::http("127.0.0.1:0").expect("start server");
  let addr = format!("http://{}", server.server_addr());
  let seen = Arc::new(Mutex::new(Vec::new()));
  let seen_clone = Arc::clone(&seen);

  thread::spawn(move || {
    for (method, path, status, body) in responses {
      let request = server
        .recv_timeout(Duration::from_secs(10))
        .expect("receive request")
        .expect("some request");
      assert_eq!(request.method(), &method);
      assert_eq!(request.url(), path);
      seen_clone
        .lock()
        .expect("lock")
        .push(format!("{} {}", request.method(), request.url()));

      let response = Response::from_string(body).with_status_code(status);
      request.respond(response).expect("respond");
    }
  });

  (addr, seen)
}

#[test]
#[serial]
fn release_non_rebuild_can_create_release_via_api() {
  let repo = create_repo();
  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"x\"); }\n").expect("write feat file");
  commit_with_date(repo.path(), "feat: add output", "2026-02-22T10:00:00Z");
  git(repo.path(), &["tag", "v0.2.0"]);

  let (base, seen) = spawn_mock_github(vec![
    (
      Method::Get,
      "/repos/o/r/releases?per_page=100".to_string(),
      200,
      "[]".to_string(),
    ),
    (Method::Post, "/repos/o/r/releases".to_string(), 201, "{}".to_string()),
  ]);

  // SAFETY: serialized test restores process env in same scope.
  unsafe { std::env::set_var("CAMBI_GITHUB_API_BASE", base) };

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd
    .current_dir(repo.path())
    .args(["release", "--owner", "o", "--repo", "r", "--token", "t"]);

  cmd.assert().success();
  assert!(seen.lock().expect("lock").len() >= 2);

  // SAFETY: serialized test restores process env in same scope.
  unsafe { std::env::remove_var("CAMBI_GITHUB_API_BASE") };
}

#[test]
#[serial]
fn release_rebuild_deletes_non_matching_and_updates_existing() {
  let repo = create_repo();
  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"x\"); }\n").expect("write feat file");
  commit_with_date(repo.path(), "feat: add output", "2026-02-22T10:00:00Z");
  git(repo.path(), &["tag", "v0.2.0"]);

  let initial = r#"[
    {"id":1,"tag_name":"v0.1.0","name":"old","body":"old"},
    {"id":2,"tag_name":"v9.9.9","name":"other","body":"other"}
  ]"#;
  let after_delete = r#"[
    {"id":1,"tag_name":"v0.1.0","name":"old","body":"old"}
  ]"#;

  let (base, seen) = spawn_mock_github(vec![
    (
      Method::Get,
      "/repos/o/r/releases?per_page=100".to_string(),
      200,
      initial.to_string(),
    ),
    (Method::Delete, "/repos/o/r/releases/2".to_string(), 204, "".to_string()),
    (
      Method::Get,
      "/repos/o/r/releases?per_page=100".to_string(),
      200,
      after_delete.to_string(),
    ),
    (
      Method::Patch,
      "/repos/o/r/releases/1".to_string(),
      200,
      "{}".to_string(),
    ),
    (Method::Post, "/repos/o/r/releases".to_string(), 201, "{}".to_string()),
  ]);

  // SAFETY: serialized test restores process env in same scope.
  unsafe { std::env::set_var("CAMBI_GITHUB_API_BASE", base) };

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd
    .current_dir(repo.path())
    .args(["release", "--rebuild", "--owner", "o", "--repo", "r", "--token", "t"]);

  cmd.assert().success();

  let calls = seen.lock().expect("lock").clone();
  assert!(calls.iter().any(|c| c.starts_with("DELETE /repos/o/r/releases/2")));

  // SAFETY: serialized test restores process env in same scope.
  unsafe { std::env::remove_var("CAMBI_GITHUB_API_BASE") };
}

#[test]
#[serial]
fn release_prerelease_flag_is_sent_to_github_api() {
  let repo = create_repo();
  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"x\"); }\n").expect("write feat file");
  commit_with_date(repo.path(), "feat: add output", "2026-02-22T10:00:00Z");
  git(repo.path(), &["tag", "v0.2.0"]);

  let server = Server::http("127.0.0.1:0").expect("start server");
  let base = format!("http://{}", server.server_addr());
  let posted_body = Arc::new(Mutex::new(String::new()));
  let posted_body_clone = Arc::clone(&posted_body);

  thread::spawn(move || {
    let list_request = server
      .recv_timeout(Duration::from_secs(10))
      .expect("receive list request")
      .expect("some list request");
    assert_eq!(list_request.method(), &Method::Get);
    assert_eq!(list_request.url(), "/repos/o/r/releases?per_page=100");
    list_request
      .respond(Response::from_string("[]").with_status_code(200))
      .expect("respond list");

    let mut post_request = server
      .recv_timeout(Duration::from_secs(10))
      .expect("receive post request")
      .expect("some post request");
    assert_eq!(post_request.method(), &Method::Post);
    assert_eq!(post_request.url(), "/repos/o/r/releases");

    let mut body = String::new();
    post_request
      .as_reader()
      .read_to_string(&mut body)
      .expect("read post body");
    *posted_body_clone.lock().expect("lock") = body;

    post_request
      .respond(Response::from_string("{}").with_status_code(201))
      .expect("respond post");
  });

  // SAFETY: serialized test restores process env in same scope.
  unsafe { std::env::set_var("CAMBI_GITHUB_API_BASE", base) };

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args([
    "release",
    "0.9.0",
    "--owner",
    "o",
    "--repo",
    "r",
    "--token",
    "t",
    "--prerelease",
  ]);

  cmd.assert().success();

  let payload: serde_json::Value = serde_json::from_str(&posted_body.lock().expect("lock")).expect("json payload");
  assert_eq!(
    payload.get("prerelease").and_then(serde_json::Value::as_bool),
    Some(true)
  );

  // SAFETY: serialized test restores process env in same scope.
  unsafe { std::env::remove_var("CAMBI_GITHUB_API_BASE") };
}
