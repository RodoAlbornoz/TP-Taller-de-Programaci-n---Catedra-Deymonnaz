use lib::coordenadas::Coordenadas;

#[derive(Debug, Clone)]
pub struct Central {
    /// El ID de la central coincide con el ID de su dron asociado
    pub id: u64,
    pub latitud: f64,
    pub longitud: f64,
    /// El rango del area de operacion del dron, se mide desde la ubicacion de su central asociada
    pub rango: f64,
}

impl Central {
    pub fn new(id: u64, latitud: f64, longitud: f64, rango: f64) -> Self {
        Central {
            id,
            latitud,
            longitud,
            rango,
        }
    }

    pub fn coordenadas(&self) -> Coordenadas {
        Coordenadas::a_partir_de_latitud_longitud(self.latitud, self.longitud)
    }
}
