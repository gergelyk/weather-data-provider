pub mod common;
pub mod aemet;
pub mod meteocat;
pub mod meteoclimatic;
pub mod weatherlink;

pub use aemet::AemetDownloader;
pub use meteocat::MeteocatDownloader;
pub use meteoclimatic::MeteoclimaticDownloader;
pub use weatherlink::WeatherlinkDownloader;
pub use common::Downloader;