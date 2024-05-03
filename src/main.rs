use dioxus::prelude::*;
use directories_next::ProjectDirs;
use std::process::{Command, Stdio};
use surrealdb::engine::remote::ws::Ws;
use surrealdb::Surreal;

static SURREAL_RUNNING: GlobalSignal<bool> = Signal::global(|| false);

fn main() {
    dioxus::launch(app);
}

pub async fn detect_surreal() {
    // Ensure Surreal is installed by trying to run `surreal --version`
    if let Err(_) = Command::new("surreal")
        .arg("--version")
        .stdout(Stdio::null())
        .spawn()
    {
        // source: https://surrealdb.com/docs/surrealdb/installation
        // the commands work on user and admin level

        #[cfg(target_os = "windows")]
        let mut command = Command::new("iwr https://windows.surrealdb.com -useb | iex");

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        let mut command = Command::new("curl -sSf https://install.surrealdb.com | sh");

        if let Err(e) = command.stdout(Stdio::null()).spawn() {
            println!("Failed to install Surreal: {}", e);
        }
    }

    if let Err(_) = Surreal::new::<Ws>("127.0.0.1:8000").await {
        *SURREAL_RUNNING.write() = false;
    } else {
        *SURREAL_RUNNING.write() = true;
    }
}

pub fn app() -> Element {
    let project_dirs = ProjectDirs::from("dev", "fokklz", "dx-surreal").unwrap();
    let db_dir = project_dirs.config_dir().join("database");

    use_coroutine(|_rx: UnboundedReceiver<()>| async move {
        detect_surreal().await;

        if !*SURREAL_RUNNING.read() {
            let mut child = Command::new("surreal")
                .arg("start")
                .args(&["--user", "root", "--pass", "root", "--auth"])
                .arg(format!("file:{}", db_dir.to_str().unwrap()))
                .stdout(Stdio::null())
                .spawn()
                .unwrap();

            *SURREAL_RUNNING.write() = true;
            tokio::spawn(async move {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        println!("exited with: {status}");
                    }
                    Ok(None) => {
                        let res = child.wait();
                        println!("result: {res:?}");
                    }
                    Err(e) => {
                        println!("error attempting to wait: {e}");
                    }
                }
            });
        }
    });

    rsx! {
        div {
            if *SURREAL_RUNNING.read() {
                "hello world"
            } else {
                "Loading..."
            }
        }
    }
}
