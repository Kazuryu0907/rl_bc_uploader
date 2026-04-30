use anyhow::Result;
use self_update::cargo_crate_version;

/// Check GitHub releases for a newer version and replace the running binary.
/// On successful update, exits the process so the user (or supervisor) restarts
/// into the new binary. If already up-to-date, returns Ok(()).
pub fn check_and_apply() -> Result<()> {
    let status = self_update::backends::github::Update::configure()
        .repo_owner("Kazuryu0907")
        .repo_name("rl_bc_uploader")
        .bin_name("rl_uploader")
        .show_download_progress(false)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;

    if status.updated() {
        tracing::info!(
            "[アップデート] v{} を適用しました — 再起動してください",
            status.version()
        );
        std::process::exit(0);
    } else {
        tracing::info!("[アップデート] 最新版です (v{})", cargo_crate_version!());
    }
    Ok(())
}
