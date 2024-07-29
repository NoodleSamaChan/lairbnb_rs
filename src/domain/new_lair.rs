use super::{LairDescription, LairImage, LairLat, LairLon, LairTitle};

pub struct NewLair {
    pub title: LairTitle,
    pub description: LairDescription,
    pub image: LairImage,
    pub lon: LairLon,
    pub lat: LairLat,
}
