pub mod aemet;
pub mod meteocat;
pub mod meteoclimatic;

pub use aemet::AemetDownloader;
pub use meteocat::MeteocatDownloader;
pub use meteoclimatic::MeteoclimaticDownloader;
