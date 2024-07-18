use crate::funcionamiento::proveer_carto::MapaCarto;
use egui::Context;
use std::{collections::HashMap, env};

use walkers::{HttpOptions, Tiles, TilesManager};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Provider {
    CartoMaps,
}

/// Opciones de HTTP para el proveedor de mapas.
fn http_options() -> HttpOptions {
    HttpOptions {
        cache: if cfg!(target_os = "android") || env::var("NO_HTTP_CACHE").is_ok() {
            None
        } else {
            Some(".cache".into())
        },
        ..Default::default()
    }
}

/// Estilos de mapa disponibles.
pub fn estilo_mapa(contexto: Context) -> HashMap<Provider, Box<dyn TilesManager + Send>> {
    let mut proveedores: HashMap<Provider, Box<dyn TilesManager + Send>> = HashMap::default();

    proveedores.insert(
        Provider::CartoMaps,
        Box::new(Tiles::with_options(
            MapaCarto {},
            http_options(),
            contexto.to_owned(),
        )),
    );

    proveedores
}
