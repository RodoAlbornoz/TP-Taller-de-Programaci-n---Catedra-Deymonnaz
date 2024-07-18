use std::collections::{HashMap, HashSet};

use lib::{camara::Camara, incidente::Incidente};

/// Estado del sistema de camaras.
pub struct Estado {
    /// Camaras conectadas al sistema.
    pub camaras: HashMap<u64, Camara>,
    /// Incidentes activos
    pub incidentes: HashMap<u64, Incidente>,
    /// Camaras lindantes a cada camara.
    pub camaras_lindantes: HashMap<u64, HashSet<u64>>,
}

impl Default for Estado {
    fn default() -> Self {
        Self::new()
    }
}

impl Estado {
    pub fn new() -> Self {
        Self {
            camaras: HashMap::new(),
            incidentes: HashMap::new(),
            camaras_lindantes: HashMap::new(),
        }
    }

    pub fn conectar_camara(&mut self, mut camara: Camara) {
        // La cámara ya existe
        if self.camara(camara.id).is_some() {
            return;
        }

        // Establece las camaras lindantes
        self.establecer_camaras_lindantes(&camara);
        // Buscamos los incidentes atendidos por la camara
        camara.incidentes_primarios = self.incidentes_en_rango(&camara);

        let binding: HashSet<u64> = HashSet::new();
        let camaras_lindantes: &HashSet<u64> =
            self.camaras_lindantes.get(&camara.id).unwrap_or(&binding);

        for id in camaras_lindantes.iter() {
            if let Some(camara_lindante) = self.camaras.get_mut(id) {
                // Agrega los incidentes secundarios a la camara, que son los incidentes primarios de las camaras lindantes
                camara
                    .incidentes_secundarios
                    .extend(camara_lindante.incidentes_primarios.clone());
                // Agrega los incidentes secundarios a la camara lindante que son los incidentes primarios de la camara
                camara_lindante
                    .incidentes_secundarios
                    .extend(camara.incidentes_primarios.clone());
            }
        }

        // Agrega la camara al estado
        self.camaras.insert(camara.id, camara);
    }

    pub fn desconectar_camara(&mut self, id: u64) -> Option<Camara> {
        if let Some(mut camara) = self.camaras.remove(&id) {
            self.restablecer_camaras_lindantes(id);
            camara.incidentes_primarios.clear();
            camara.incidentes_secundarios.clear();
            Some(camara)
        } else {
            None
        }
    }

    pub fn cargar_incidente(&mut self, incidente: Incidente) {
        // Primero finaliza el incidente ya que podia ya existir previamente
        self.finalizar_incidente(incidente.id);

        self.incidentes.insert(incidente.id, incidente.clone());

        for id_camara in self.camaras_en_rango(&incidente) {
            if let Some(camara) = self.camaras.get_mut(&id_camara) {
                camara.incidentes_primarios.insert(incidente.id);
            }

            for id_camara_lindante in self
                .camaras_lindantes
                .get(&id_camara)
                .unwrap_or(&HashSet::new())
            {
                if let Some(camara_lindante) = self.camaras.get_mut(id_camara_lindante) {
                    camara_lindante.incidentes_secundarios.insert(incidente.id);
                }
            }
        }
    }

    pub fn finalizar_todos_los_incidentes(&mut self) {
        let incidentes: Vec<u64> = self.incidentes.keys().copied().collect();
        for id in incidentes {
            self.finalizar_incidente(id);
        }
    }

    pub fn finalizar_incidente(&mut self, id: u64) -> Option<Incidente> {
        if let Some(incidente) = self.incidentes.remove(&id) {
            for id_camara in self.camaras_en_rango(&incidente) {
                if let Some(camara) = self.camaras.get_mut(&id_camara) {
                    camara.incidentes_primarios.remove(&id);
                }

                for id_camara_lindante in self
                    .camaras_lindantes
                    .get(&id_camara)
                    .unwrap_or(&HashSet::new())
                {
                    if let Some(camara_lindante) = self.camaras.get_mut(id_camara_lindante) {
                        camara_lindante.incidentes_secundarios.remove(&id);
                    }
                }
            }

            Some(incidente)
        } else {
            None
        }
    }

    pub fn modificar_ubicacion_camara(&mut self, id: u64, latitud: f64, longitud: f64) {
        // Hasta aca borramos la camara, y re acomodamos las lindantes
        if let Some(mut camara) = self.desconectar_camara(id) {
            camara.latitud = latitud;
            camara.longitud = longitud;
            self.conectar_camara(camara);
        }
    }

    pub fn modificar_rango_camara(&mut self, id: u64, rango: f64) {
        if let Some(mut camara) = self.desconectar_camara(id) {
            camara.rango = rango;
            self.conectar_camara(camara);
        }
    }

    fn establecer_camaras_lindantes(&mut self, camara: &Camara) {
        // Calcula las camaras lindantes a la camara dada
        let camaras_lindantes: HashSet<u64> = self
            .camaras
            .values()
            .filter(|otra_camara: &&Camara| {
                camara.posicion().distancia(&otra_camara.posicion())
                    < camara.rango + otra_camara.rango
                    && otra_camara.id != camara.id
            })
            .map(|c: &Camara| c.id)
            .collect();

        // Almacena la camara dada como lindante para las camaras lindantes
        for id in camaras_lindantes.iter() {
            self.camaras_lindantes
                .entry(*id)
                .or_default()
                .insert(camara.id);
        }

        // Almacena las camaras lindantes para la camara dada
        self.camaras_lindantes.insert(camara.id, camaras_lindantes);
    }

    fn restablecer_camaras_lindantes(&mut self, id: u64) {
        // En las camaras lindantes a la camara con ese id, elimina a la
        // camara con ese id como lindante
        if let Some(camaras_lindantes) = self.camaras_lindantes.remove(&id) {
            for id_lindante in camaras_lindantes.iter() {
                if let Some(camaras_lindantes_de_camara_lindante) =
                    self.camaras_lindantes.get_mut(id_lindante)
                {
                    camaras_lindantes_de_camara_lindante.remove(&id);
                }
            }
        }
    }

    /// Incidentes que están en el rango de una cámara
    pub fn incidentes_en_rango(&self, camara: &Camara) -> HashSet<u64> {
        let mut incidentes: HashSet<u64> = HashSet::new();
        for incidente in self.incidentes.values() {
            if incidente.posicion().distancia(&camara.posicion()) < camara.rango {
                incidentes.insert(incidente.id);
            }
        }
        incidentes
    }

    pub fn camaras_en_rango(&self, incidente: &Incidente) -> HashSet<u64> {
        let mut camaras: HashSet<u64> = HashSet::new();

        for camara in self.camaras.values() {
            if incidente.posicion().distancia(&camara.posicion()) < camara.rango {
                camaras.insert(camara.id);
            }
        }
        camaras
    }

    /// Devuelve una referencia a la camara con el id dado.
    pub fn camara(&self, id: u64) -> Option<&Camara> {
        self.camaras.get(&id)
    }

    /// Devuelve un vector de camaras
    pub fn camaras(&self) -> Vec<&Camara> {
        self.camaras.values().collect()
    }
}

#[cfg(test)]
mod test {
    use lib::coordenadas::Coordenadas;

    use super::*;

    #[test]
    fn test_conectar_camara() {
        let mut estado = Estado::new();
        let camara = Camara::new(1, 0.0, 0.0, 1.0);
        estado.conectar_camara(camara.clone());
        assert_eq!(estado.camaras.get(&1), Some(&camara));
    }

    #[test]
    fn test_desconectar_camara() {
        let mut estado = Estado::new();
        let camara = Camara::new(1, 0.0, 0.0, 1.0);
        estado.conectar_camara(camara.clone());
        assert_eq!(estado.desconectar_camara(1), Some(camara));
        assert_eq!(estado.camaras.get(&1), None);
    }

    #[test]
    fn test_cargar_incidente() {
        let mut estado = Estado::new();
        let incidente = Incidente::new(1, "Incidente".to_string(), 0.0, 0.0, 0);
        estado.cargar_incidente(incidente.clone());
        assert_eq!(estado.incidentes.get(&1), Some(&incidente));
    }

    #[test]
    fn test_finalizar_incidente() {
        let mut estado = Estado::new();
        let incidente = Incidente::new(1, "Incidente".to_string(), 0.0, 0.0, 0);
        estado.cargar_incidente(incidente.clone());
        assert_eq!(estado.finalizar_incidente(1), Some(incidente));
        assert_eq!(estado.incidentes.get(&1), None);
    }

    #[test]
    fn test_modificar_ubicacion_camara() {
        let mut estado = Estado::new();
        let mut camara = Camara::new(1, 0.0, 0.0, 1.0);
        estado.conectar_camara(camara.clone());
        estado.modificar_ubicacion_camara(1, 1.0, 1.0);
        camara.latitud = 1.0;
        camara.longitud = 1.0;
        assert_eq!(estado.camaras.get(&1), Some(&camara));
    }

    #[test]
    fn test_modificar_rango_camara() {
        let mut estado = Estado::new();
        let mut camara = Camara::new(1, 0.0, 0.0, 1.0);
        estado.conectar_camara(camara.clone());
        estado.modificar_rango_camara(1, 2.0);
        camara.rango = 2.0;
        assert_eq!(estado.camaras.get(&1), Some(&camara));
    }

    #[test]
    fn cagar_incidente_en_rango_de_camara() {
        let mut estado = Estado::new();
        let camara = Camara::new(1, 0.0, 0.0, 1.0);
        estado.conectar_camara(camara.clone());
        let incidente = Incidente::new(1, "Incidente".to_string(), 0.0, 0.0, 0);
        estado.cargar_incidente(incidente.clone());
        assert_eq!(
            estado.camaras.get(&1).unwrap().incidentes_primarios.get(&1),
            Some(&1)
        );
    }

    #[test]
    fn cargar_camara_en_rango_incidente() {
        let mut estado = Estado::new();
        let incidente = Incidente::new(1, "Incidente".to_string(), 0.0, 0.0, 0);
        estado.cargar_incidente(incidente.clone());
        let camara = Camara::new(1, 0.0, 0.0, 1.0);
        estado.conectar_camara(camara.clone());

        assert_eq!(
            estado
                .camaras_en_rango(estado.incidentes.get(&1).unwrap())
                .get(&1),
            Some(&1)
        );
    }

    #[test]
    fn cargar_incidente_activar_camara_lindante() {
        let mut estado = Estado::new();
        let rango = Coordenadas::a_partir_de_latitud_longitud(0.0, 0.0)
            .distancia(&Coordenadas::a_partir_de_latitud_longitud(0.0, 1.0))
            * 0.55;
        let camara = Camara::new(1, 0.0, 0.0, rango);
        estado.conectar_camara(camara.clone());

        let camara_lindante = Camara::new(2, 0.0, 1.0, rango);
        estado.conectar_camara(camara_lindante.clone());

        let incidente = Incidente::new(1, "Incidente".to_string(), 0.0, 0.0, 0);
        estado.cargar_incidente(incidente.clone());
        assert_eq!(
            estado
                .camaras
                .get(&2)
                .unwrap()
                .incidentes_secundarios
                .get(&1),
            Some(&1)
        );
    }
}
