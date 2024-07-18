use lib::coordenadas::Coordenadas;

#[derive(Debug, Clone)]
pub struct Desplazamiento {
    pub latitud: f64,
    pub longitud: f64,
    pub velocidad: u64,
}

impl Desplazamiento {
    pub fn new(latitud: f64, longitud: f64, velocidad: u64) -> Self {
        Desplazamiento {
            latitud,
            longitud,
            velocidad,
        }
    }

    pub fn coordenadas(&self) -> Coordenadas {
        Coordenadas::a_partir_de_latitud_longitud(self.latitud, self.longitud)
    }

    pub fn actualizar_latitud_longitud(&mut self, latitud: f64, longitud: f64) {
        self.latitud = latitud;
        self.longitud = longitud;
    }

    pub fn interpolaciones(&self, latitud_destino: f64, longitud_destino: f64) -> Vec<Coordenadas> {
        let coordenadas_actuales: Coordenadas = self.coordenadas();
        let coordenadas_destino: Coordenadas =
            Coordenadas::a_partir_de_latitud_longitud(latitud_destino, longitud_destino);
        let segundos_hasta_destino: usize =
            self.segundos_hasta_destino(latitud_destino, longitud_destino);

        // Defino que la cantidad de pasos para la interpolacion sera igual a la cantidad de segundos
        // que se tarda en llegar al destino
        coordenadas_actuales.interpolar(&coordenadas_destino, segundos_hasta_destino)
    }

    pub fn segundos_hasta_destino(&self, latitud_destino: f64, longitud_destino: f64) -> usize {
        let coordenadas_actuales: Coordenadas = self.coordenadas();
        let coordenadas_destino: Coordenadas =
            Coordenadas::a_partir_de_latitud_longitud(latitud_destino, longitud_destino);
        let distancia: f64 = coordenadas_actuales.distancia(&coordenadas_destino);

        // La distancia dividida la velocidad nos da el tiempo que se tarda en llegar al destino
        // en segundos
        (distancia / (self.velocidad as f64)) as usize
    }

    /// Devuelve true si es posible ir y volver del destino con la bateria actual, false en caso
    /// contrario, tomando varios parametros para el calculo.
    pub fn esta_en_el_alcance_maximo(
        &self,
        latitud_destino: f64,
        longitud_destino: f64,
        bateria_duracion_actual: i64,
        tiempo_atender_incidente: i64,
    ) -> bool {
        let segundos_hasta_destino =
            self.segundos_hasta_destino(latitud_destino, longitud_destino) as i64;

        let tiempo_desplazamiento = segundos_hasta_destino * 2;

        bateria_duracion_actual - tiempo_desplazamiento - tiempo_atender_incidente - 15 > 0
    }

    /// Devuelve true si es alcanzable (o sea, ir y volver sin problemas), false en caso contrario,
    /// tomando varios parametros para el calculo. Si es false, el incidente no queda cargado en el
    /// dron.
    pub fn es_alcanzable(
        &self,
        latitud_destino: f64,
        longitud_destino: f64,
        bateria_duracion_total: i64,
        tiempo_atender_incidente: i64,
    ) -> bool {
        let segundos_hasta_destino =
            self.segundos_hasta_destino(latitud_destino, longitud_destino) as i64;

        let tiempo_desplazamiento = segundos_hasta_destino * 2;

        bateria_duracion_total - tiempo_desplazamiento - tiempo_atender_incidente - 15 > 0
    }
}
