use walkers::{
    sources::{Attribution, TileSource},
    TileId,
};

pub struct MapaCarto;

impl TileSource for MapaCarto {
    fn tile_url(&self, id_tile: TileId) -> String {
        format!(
            "https://d.basemaps.cartocdn.com/light_all/{}/{}/{}@2x.png",
            id_tile.zoom, id_tile.x, id_tile.y
        )
    }

    fn attribution(&self) -> Attribution {
        Attribution {
            text: "CARTO Attribution",
            url: "https://carto.com/attribution",
            logo_light: None,
            logo_dark: None,
        }
    }
}
