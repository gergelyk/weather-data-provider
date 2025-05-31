use anyhow::anyhow;
use leptos::leptos_dom::logging::console_error;
use leptos::prelude::*;

pub fn get_root_url() -> anyhow::Result<String> {
    let loc = location();
    let href = loc.href().map_err(|e| {
        let e = format!("{:?}", e);
        anyhow!(e).context("Failed to get href")
    })?;
    let pathname = loc.pathname().map_err(|e| {
        let e = format!("{:?}", e);
        anyhow!(e).context("Failed to get pathname")
    })?;
    let root = href
        .strip_suffix(&pathname)
        .ok_or_else(|| anyhow!("Pathname not in tact with href"))?;
    Ok(root.to_string())
}
pub fn log_anyhow_error(e: anyhow::Error) {
    console_error(&format!("{:?}", e));
}
