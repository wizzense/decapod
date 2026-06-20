use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

fn run_decapod(dir: &Path, args: &[&str], envs: &[(&str, &str)]) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_decapod"));
    cmd.current_dir(dir).args(args);
    cmd.env_remove("DECAPOD_VALIDATE_SKIP_GIT_GATES");
    cmd.env_remove("DECAPOD_VALIDATE_SKIP_TOOLING_GATES");
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("run decapod")
}

fn setup_repo() -> (TempDir, PathBuf, String) {
    let tmp = TempDir::new().expect("tmpdir");
    let repo_dir = tmp.path().to_path_buf();

    let init = Command::new("git")
        .current_dir(&repo_dir)
        .args(["init", "-b", "master"])
        .output()
        .expect("git init");
    assert!(init.status.success(), "git init failed");

    let out = run_decapod(&repo_dir, &["init", "--force"], &[]);
    assert!(
        out.status.success(),
        "decapod init failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let config_name = Command::new("git")
        .current_dir(&repo_dir)
        .args(["config", "user.name", "Test User"])
        .output()
        .expect("git config user.name");
    assert!(config_name.status.success(), "git config user.name failed");

    let config_email = Command::new("git")
        .current_dir(&repo_dir)
        .args(["config", "user.email", "test@example.com"])
        .output()
        .expect("git config user.email");
    assert!(
        config_email.status.success(),
        "git config user.email failed"
    );

    let add = Command::new("git")
        .current_dir(&repo_dir)
        .args(["add", "."])
        .output()
        .expect("git add");
    assert!(add.status.success(), "git add failed");

    let commit = Command::new("git")
        .current_dir(&repo_dir)
        .args(["commit", "-m", "init"])
        .output()
        .expect("git commit");
    assert!(commit.status.success(), "git commit failed");

    let worktree_dir = repo_dir
        .join(".decapod")
        .join("workspaces")
        .join("test-worktree");
    fs::create_dir_all(worktree_dir.parent().unwrap()).unwrap();

    let worktree = Command::new("git")
        .current_dir(&repo_dir)
        .args([
            "worktree",
            "add",
            "-b",
            "agent/test/git-push-pr",
            worktree_dir
                .to_str()
                .expect("tempdir path should be valid unicode"),
            "HEAD",
        ])
        .output()
        .expect("git worktree add");
    assert!(worktree.status.success(), "git worktree add failed");

    let acquire = run_decapod(
        &worktree_dir,
        &["session", "acquire"],
        &[("DECAPOD_AGENT_ID", "unknown")],
    );
    assert!(
        acquire.status.success(),
        "session acquire failed: {}",
        String::from_utf8_lossy(&acquire.stderr)
    );
    let password = String::from_utf8_lossy(&acquire.stdout)
        .lines()
        .find_map(|line| {
            line.strip_prefix("Password: ")
                .map(|s| s.trim().to_string())
        })
        .expect("session password in output");

    (tmp, worktree_dir, password)
}

#[test]
fn git_push_pr_gate_warns_when_unpushed() {
    let (_tmp, dir, password) = setup_repo();

    let validate = run_decapod(
        &dir,
        &["validate", "-v"],
        &[
            ("DECAPOD_AGENT_ID", "unknown"),
            ("DECAPOD_SESSION_PASSWORD", &password),
            ("DECAPOD_CONTAINER", "1"),
        ],
    );

    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&validate.stdout),
        String::from_utf8_lossy(&validate.stderr)
    );
    assert!(
        combined.contains("has unpushed commits or does not exist on origin"),
        "expected git-push-pr gate warning marker, got: {combined}"
    );
}
