use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use sqlx::SqlitePool;

/// Test helper to spawn the backend server binary on a free port with a temp DB
pub struct TestServer {
    pub base_url: String,
    child: Child,
    _db_path: PathBuf,
}

impl TestServer {
    pub async fn spawn() -> anyhow::Result<Self> {
        // Pick a free port
        let port = portpicker::pick_unused_port().expect("no free ports available");
        let host = "127.0.0.1";

        // Temp DB file
        let tmp = tempfile::NamedTempFile::new()?;
        let db_path = tmp.into_temp_path().to_path_buf();
        let database_url = format!("sqlite://{}", db_path.display());

        // Resolve binary path
        let bin = resolve_binary_path();
        anyhow::ensure!(bin.exists(), "backend binary not found at {:?}", bin);

        // Spawn server
        let child = Command::new(bin)
            .env("HOST", host)
            .env("PORT", port.to_string())
            .env("DATABASE_URL", database_url)
            .env("RUST_LOG", "debug")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        let base = format!("http://{}:{}", host, port);
        // Wait for /health to be ready
        wait_for_health(&base, Duration::from_secs(10)).await?;

        Ok(Self { base_url: base, child, _db_path: db_path })
    }

    /// Spawn server with additional environment variables (e.g., AI_API_BASE)
    pub async fn spawn_with_env(extra_env: Vec<(String, String)>) -> anyhow::Result<Self> {
        // Pick a free port
        let port = portpicker::pick_unused_port().expect("no free ports available");
        let host = "127.0.0.1";

        // Temp DB file
        let tmp = tempfile::NamedTempFile::new()?;
        let db_path = tmp.into_temp_path().to_path_buf();
        let database_url = format!("sqlite://{}", db_path.display());

        // Resolve binary path
        let bin = resolve_binary_path();
        anyhow::ensure!(bin.exists(), "backend binary not found at {:?}", bin);

        let mut cmd = Command::new(bin);
        cmd.env("HOST", host)
            .env("PORT", port.to_string())
            .env("DATABASE_URL", database_url)
            .env("RUST_LOG", "debug");
        for (k, v) in extra_env.into_iter() {
            cmd.env(k, v);
        }

        let child = cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit()).spawn()?;

        let base = format!("http://{}:{}", host, port);
        // Wait for /health to be ready
        wait_for_health(&base, Duration::from_secs(10)).await?;

        Ok(Self { base_url: base, child, _db_path: db_path })
    }

    /// Database URL (sqlite://...)
    pub fn database_url(&self) -> String {
        format!("sqlite://{}", self._db_path.display())
    }

    /// Connect to the same SQLite database the server is using
    pub async fn connect_db(&self) -> anyhow::Result<SqlitePool> {
        let pool = SqlitePool::connect(&self.database_url()).await?;
        Ok(pool)
    }

    /// Get the database file path
    pub fn db_path(&self) -> &PathBuf {
        &self._db_path
    }

    /// Spawn server with a specific database file (for persistence testing)
    pub async fn spawn_with_db_path(db_path: PathBuf) -> anyhow::Result<Self> {
        // Pick a free port
        let port = portpicker::pick_unused_port().expect("no free ports available");
        let host = "127.0.0.1";

        let database_url = format!("sqlite://{}", db_path.display());

        // Resolve binary path
        let bin = resolve_binary_path();
        anyhow::ensure!(bin.exists(), "backend binary not found at {:?}", bin);

        // Spawn server
        let child = Command::new(bin)
            .env("HOST", host)
            .env("PORT", port.to_string())
            .env("DATABASE_URL", database_url)
            .env("RUST_LOG", "debug")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        let base = format!("http://{}:{}", host, port);
        // Wait for /health to be ready
        wait_for_health(&base, Duration::from_secs(10)).await?;

        Ok(Self { base_url: base, child, _db_path: db_path })
    }

    /// Stop the server gracefully
    pub fn stop(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        // Try to terminate the child process
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

async fn wait_for_health(base: &str, timeout: Duration) -> anyhow::Result<()> {
    let url = format!("{}/health", base);
    let start = Instant::now();
    loop {
        if start.elapsed() > timeout {
            anyhow::bail!("server did not become ready at {} within {:?}", url, timeout)
        }
        match reqwest::get(&url).await {
            Ok(resp) if resp.status().as_u16() == 200 => return Ok(()),
            _ => tokio::time::sleep(Duration::from_millis(200)).await,
        }
    }
}

fn resolve_binary_path() -> PathBuf {
    // Try Cargo-provided env vars first (hyphen and underscore forms)
    if let Ok(p) = std::env::var("CARGO_BIN_EXE_paper-gloss") {
        return PathBuf::from(p);
    }
    if let Ok(p) = std::env::var("CARGO_BIN_EXE_paper_gloss") {
        return PathBuf::from(p);
    }
    // Fallback to target/debug
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("target");
    p.push("debug");
    p.push(if cfg!(windows) { "paper-gloss.exe" } else { "paper-gloss" });
    p
}

/// Helper to build API base URL with "/api" prefix per OpenAPI
pub fn api(base: &str) -> String {
    format!("{}/api", base)
}
