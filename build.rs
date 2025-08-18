use std::env;
use std::fs;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    // 获取 Git 短 commit
    let git_commit = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_owned())
        .unwrap_or_else(|| "unknown".into());

    // 获取当前分支
    let git_branch = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_owned())
        .unwrap_or_else(|| "unknown".into());

    // 获取构建时间（UTC）
    let build_time = chrono::Utc::now().to_rfc3339();

    // 生成一个 Rust 文件写入常量
    fs::write(
        format!("{}/build_info.rs", out_dir),
        format!(
            r#"
                pub const VERSION: &str = "{}";
                pub const GIT_COMMIT: &str = "{}";
                pub const GIT_BRANCH: &str = "{}";
                pub const BUILD_TIME: &str = "{}";
            "#,
            env::var("CARGO_PKG_VERSION").unwrap(),
            git_commit,
            git_branch,
            build_time
        ),
    )
    .expect("Failed to write build info");

    // 当 git 变化时重新构建
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
}