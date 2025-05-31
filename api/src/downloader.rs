use crate::measurements::Measurements;
pub trait Downloader {
    fn base_url(&self) -> String;
    async fn try_download(&self, url: &str) -> anyhow::Result<Measurements>;

    async fn download(&self, url: &str) -> Measurements {
        let payload = self.try_download(url).await;
        match payload {
            Ok(payload) => {
                log::info!("Downloaded: {}", url);
                payload
            }
            Err(ref e) => {
                log::error!("{} while downloading: {}", e, url);
                Measurements::default()
            }
        }
    }
}
