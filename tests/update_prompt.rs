//! The once-a-day update offer, driven through the binary from a fake managed layout: the test
//! binary is copied into `versions/v<crate version>/bin` so `locate_install` recognises it, a
//! local fake server plays GitHub, and the "new version" it serves is a shell script that
//! records how it was re-executed. Nothing touches the network.
#![cfg(unix)]

use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use flate2::Compression;
use flate2::write::GzEncoder;
use resolve_merge_conflicts::selfupdate::download::sha256_hex;
use resolve_merge_conflicts::selfupdate::host_target;

const RUNNING: &str = concat!("v", env!("CARGO_PKG_VERSION"));
const NEWER: &str = "v99.0.0";
/// What the fake newer binary prints when it runs.
const MARKER: &str = "REEXEC_MARKER";

struct Layout {
    _temp: tempfile::TempDir,
    root: PathBuf,
    exe: PathBuf,
}

/// versions/<RUNNING>/bin holds a copy of the real test binary, which is also `current`.
fn layout() -> Layout {
    let temp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(temp.path()).unwrap().join("root");
    let bin = root.join("versions").join(RUNNING).join("bin");
    fs::create_dir_all(&bin).unwrap();
    let exe = bin.join("resolve-merge-conflicts");
    fs::copy(env!("CARGO_BIN_EXE_resolve-merge-conflicts"), &exe).unwrap();
    std::os::unix::fs::symlink(root.join("versions").join(RUNNING), root.join("current")).unwrap();
    Layout {
        _temp: temp,
        root,
        exe,
    }
}

type Routes = Arc<Mutex<HashMap<String, Vec<u8>>>>;

/// Serves the routes it is given with 200, everything else 404. One request per connection.
fn fake_server(routes: HashMap<String, Vec<u8>>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let routes: Routes = Arc::new(Mutex::new(routes));
    thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            let routes = routes.clone();
            thread::spawn(move || {
                stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
                let mut buf = Vec::new();
                let mut chunk = [0u8; 1024];
                while let Ok(n) = stream.read(&mut chunk) {
                    if n == 0 {
                        break;
                    }
                    buf.extend_from_slice(&chunk[..n]);
                    if buf.windows(4).any(|w| w == b"\r\n\r\n") {
                        break;
                    }
                }
                let request = String::from_utf8_lossy(&buf);
                let path = request
                    .lines()
                    .next()
                    .and_then(|l| l.split_whitespace().nth(1))
                    .unwrap_or("/")
                    .to_owned();
                let (status, body) = match routes.lock().unwrap().get(&path) {
                    Some(body) => ("200 OK", body.clone()),
                    None => ("404 Not Found", b"not found".to_vec()),
                };
                let _ = write!(
                    stream,
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(&body);
            });
        }
    });
    base
}

/// A fake GitHub whose latest release is [`NEWER`], serving a shell script as its asset. The
/// script prints its arguments and the re-exec marker variable, so a run that continued on the
/// "new version" is visible in stdout.
fn github_offering_newer() -> String {
    let target = host_target().expect("test host is a built target");
    let script = format!(
        "#!/bin/sh\necho {MARKER} \"$@\"\necho REEXEC=${{RESOLVE_MERGE_CONFLICTS_REEXEC:-unset}}\nexit 0\n"
    );
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(script.as_bytes()).unwrap();
    let archive = encoder.finish().unwrap();

    let mut routes = HashMap::new();
    routes.insert(
        "/repos/CodeVachon/resolve-merge-conflict/releases/latest".to_owned(),
        format!("{{\"tag_name\":\"{NEWER}\",\"draft\":false,\"prerelease\":false}}").into_bytes(),
    );
    routes.insert(format!("/download/{}", target.asset_name), archive.clone());
    routes.insert(
        "/download/checksums.txt".to_owned(),
        format!("{}  {}\n", sha256_hex(&archive), target.asset_name).into_bytes(),
    );
    fake_server(routes)
}

/// Run `exe` as the interactive command with `--yes`, pointed at the fake GitHub. `CI` is
/// removed because the offer is off under it, and GitHub Actions sets it.
fn run_interactive(exe: &Path, base: &str, extra_env: &[(&str, &str)]) -> (i32, String, String) {
    let cwd = tempfile::tempdir().unwrap();
    let mut command = Command::new(exe);
    command
        .args(["--yes", "-a", "claude", "-c"])
        .arg(cwd.path())
        .env_remove("CI")
        .env_remove("RESOLVE_MERGE_CONFLICTS_NO_UPDATE_CHECK")
        .env_remove("RESOLVE_MERGE_CONFLICTS_REEXEC")
        .env("RESOLVE_MERGE_CONFLICTS_API_BASE", base)
        .env(
            "RESOLVE_MERGE_CONFLICTS_DOWNLOAD_BASE",
            format!("{base}/download"),
        );
    for (k, v) in extra_env {
        command.env(k, v);
    }
    let output = output_retrying(&mut command).expect("binary runs");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// Run a command, retrying on `ETXTBSY` ("Text file busy", raw os error 26).
///
/// `layout`'s exe is a fresh `fs::copy` of the running test binary. On Linux, `execve` can very
/// briefly still see such a file as open for writing even after the copy's own file handle has
/// closed — a known kernel/VFS race, sharper under CI's containerised filesystems, that has
/// nothing to do with test correctness. Retrying a few times with a short sleep is the standard
/// mitigation; any other error, or running out of attempts, propagates immediately.
fn output_retrying(command: &mut Command) -> std::io::Result<std::process::Output> {
    const ETXTBSY: i32 = 26;
    for attempt in 0..20 {
        match command.output() {
            Err(error) if error.raw_os_error() == Some(ETXTBSY) && attempt < 19 => {
                thread::sleep(Duration::from_millis(25));
            }
            result => return result,
        }
    }
    unreachable!("loop always returns before exhausting its range")
}

fn current_of(root: &Path) -> String {
    fs::read_link(root.join("current"))
        .unwrap()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[test]
fn accepted_offer_installs_repoints_and_continues_the_run_on_the_new_version() {
    let layout = layout();
    let base = github_offering_newer();

    let (code, out, err) = run_interactive(&layout.exe, &base, &[]);
    assert_eq!(code, 0, "stdout:\n{out}\nstderr:\n{err}");

    // The upgrade ran with its usual lines...
    assert!(
        out.contains(&format!("downloading resolve-merge-conflicts {NEWER}")),
        "{out}"
    );
    assert!(out.contains("✓ checksum verified"), "{out}");
    assert!(out.contains(&format!("✓ installed {NEWER}")), "{out}");
    assert!(
        out.contains(&format!("now using {NEWER} (was {RUNNING})")),
        "{out}"
    );
    assert_eq!(current_of(&layout.root), NEWER);

    // ...and the process re-executed on the new binary with the same arguments and the guard set.
    let marker_line = out
        .lines()
        .find(|l| l.starts_with(MARKER))
        .unwrap_or_else(|| panic!("no re-exec marker in:\n{out}"));
    assert!(marker_line.contains("--yes -a claude -c "), "{marker_line}");
    assert!(out.contains("REEXEC=1"), "{out}");
    // The rest of the original run never happened in this process.
    assert!(!err.contains("not inside a git repository"), "{err}");

    let record: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(layout.root.join("update-check.json")).unwrap())
            .unwrap();
    assert_eq!(record["latest"], NEWER);
}

#[test]
fn within_a_day_of_the_last_attempt_nothing_is_offered() {
    let layout = layout();
    let base = github_offering_newer();
    fs::write(
        layout.root.join("update-check.json"),
        format!(
            "{{\"lastAttemptAt\":{},\"latest\":\"{NEWER}\"}}\n",
            now_millis() - 60_000
        ),
    )
    .unwrap();

    let (_, out, err) = run_interactive(&layout.exe, &base, &[]);
    assert!(!out.contains("downloading"), "{out}");
    assert!(!out.contains(MARKER), "{out}");
    // The run itself proceeded (it reaches the repository check and fails there, on this version).
    assert!(err.contains("not inside a git repository"), "{err}");
    assert_eq!(current_of(&layout.root), RUNNING);
}

#[test]
fn opt_out_variable_and_reexec_guard_suppress_the_offer() {
    let layout = layout();
    let base = github_offering_newer();

    for env in [
        ("RESOLVE_MERGE_CONFLICTS_NO_UPDATE_CHECK", "1"),
        ("RESOLVE_MERGE_CONFLICTS_REEXEC", "1"),
    ] {
        let (_, out, err) = run_interactive(&layout.exe, &base, &[env]);
        assert!(!out.contains("downloading"), "{env:?}: {out}");
        assert!(
            err.contains("not inside a git repository"),
            "{env:?}: {err}"
        );
        assert!(
            !layout.root.join("update-check.json").exists(),
            "{env:?} must not even look"
        );
    }
    assert_eq!(current_of(&layout.root), RUNNING);
}

#[test]
fn an_unmanaged_binary_is_never_offered_an_update() {
    let base = github_offering_newer();
    let (_, out, err) = run_interactive(
        Path::new(env!("CARGO_BIN_EXE_resolve-merge-conflicts")),
        &base,
        &[],
    );
    assert!(!out.contains("downloading"), "{out}");
    assert!(!out.contains(MARKER), "{out}");
    assert!(err.contains("not inside a git repository"), "{err}");
}

#[test]
fn an_unreachable_release_api_is_silent_and_the_run_proceeds() {
    let layout = layout();
    // A bound-then-dropped port: connection refused, quickly.
    let refused = {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        format!("http://{}", listener.local_addr().unwrap())
    };

    let (_, out, err) = run_interactive(&layout.exe, &refused, &[]);
    assert!(!out.contains("downloading"), "{out}");
    assert!(!err.contains("warning"), "{err}");
    assert!(err.contains("not inside a git repository"), "{err}");
    // The failed attempt still counts for the day.
    let record: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(layout.root.join("update-check.json")).unwrap())
            .unwrap();
    assert!(record["lastAttemptAt"].as_u64().unwrap() > 0);
    assert!(record["latest"].is_null());
}
