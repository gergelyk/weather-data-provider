pub mod common;
pub mod aemet;
pub mod meteocat;
pub mod meteoclimatic;
pub mod weatherlink;
pub mod openwindmap;

pub use aemet::AemetDownloader;
pub use meteocat::MeteocatDownloader;
pub use meteoclimatic::MeteoclimaticDownloader;
pub use weatherlink::WeatherlinkDownloader;
pub use openwindmap::OpenWindMapDownloader;

pub use common::Downloader;