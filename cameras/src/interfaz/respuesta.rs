use lib::camara::Camara;

/// Respuesta a un comando.
pub enum Respuesta {
    Ok,
    Error(String),
    Camaras(Vec<Camara>),
    Camara(Camara),
    Ayuda,
}

impl Respuesta {
    /// Devuelve la respuesta como un string.
    pub fn como_string(&self) -> String {
        match self {
            Respuesta::Ok => "Ok".to_string(),
            Respuesta::Error(error) => format!("Error: {}", error),
            Respuesta::Camaras(camaras) => self.camaras_string(camaras),
            Respuesta::Ayuda => "conectar <ID> <Lat> <Lon> <Rango>\ndesconectar <ID>\nlistar\nmodificar ubicacion <ID> <Lat> <Lon>\nmodificar rango <ID> <Rango>\nayuda".to_string(),
            Respuesta::Camara(camara) => self.camara_string(camara),
        }
    }

    /// Devuelve la representación en string de una cámara.
    fn camara_string(&self, camara: &Camara) -> String {
        let mut estado: &str = "Modo ahorro";

        if camara.activa() {
            estado = "Activa";
        }

        format!(
            "ID: {}, Lat: {}, Lon: {}, Rango: {} Estado: {}. Incidentes primarios: {}, Incidentes secundarios: {}",
            camara.id, camara.latitud, camara.longitud, camara.rango, estado, camara.incidentes_primarios.len(),
            camara.incidentes_secundarios.len()
        )
    }

    /// Devuelve la representacion en string de todas las camaras.
    /// Llama en bucle a camara_string.
    fn camaras_string(&self, camaras: &[Camara]) -> String {
        let lineas: Vec<String> = camaras
            .iter()
            .map(|c| self.camara_string(c))
            .collect::<Vec<String>>();
        lineas.join("\n")
    }
}
