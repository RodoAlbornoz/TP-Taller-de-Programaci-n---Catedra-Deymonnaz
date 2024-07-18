use drone::dron::Dron;
use egui::ahash::HashMap;

use lib::{camara::Camara, incidente::Incidente};

/// Estado de la aplicación. Todos los incidentes, cámaras y drones que hay que mostrar.
#[derive(Clone, Debug)]
pub struct Estado {
    camaras: HashMap<u64, Camara>,
    incidentes: HashMap<u64, Incidente>,
    drones: HashMap<u64, Dron>,
    pub conectado: bool,
    pub mensaje_error: Option<String>,
}

impl Default for Estado {
    fn default() -> Self {
        Self::new()
    }
}

impl Estado {
    pub fn new() -> Self {
        Estado {
            camaras: HashMap::default(),
            incidentes: HashMap::default(),
            drones: HashMap::default(),
            conectado: false,
            mensaje_error: None,
        }
    }

    /// Agrega un incidente en el estado.
    /// Se usa cuando se genera un incidente.
    pub fn cargar_incidente(&mut self, incidente: Incidente) {
        self.incidentes.insert(incidente.id, incidente);
    }

    /// Elimina un incidente en el estado.
    /// Se usa cuando se finaliza un incidente.
    pub fn finalizar_incidente(&mut self, id: &u64) -> Option<Incidente> {
        self.incidentes.remove(id)
    }

    /// Agrega una cámara en el estado.
    /// Se usa cuando se conecta una cámara.
    pub fn conectar_camara(&mut self, camara: Camara) {
        self.camaras.insert(camara.id, camara);
    }

    /// Elimina una cámara en el estado.
    /// Se usa cuando se desconecta una cámara.
    pub fn desconectar_camara(&mut self, id: &u64) -> Option<Camara> {
        self.camaras.remove(id)
    }

    /// Agrega un dron en el estado.
    /// Se usa cuando se conecta un dron.
    pub fn conectar_dron(&mut self, dron: Dron) {
        self.drones.insert(dron.id, dron);
    }

    /// Elimina un dron en el estado.
    /// Se usa cuando se desconecta un dron.
    pub fn desconectar_dron(&mut self, id: &u64) -> Option<Dron> {
        self.drones.remove(id)
    }

    /// Muestra todos los incidentes activos por orden de inicio.
    pub fn incidentes(&self) -> Vec<Incidente> {
        let mut v: Vec<Incidente> = self.incidentes.values().cloned().collect();
        v.sort_by(|a, b| b.inicio.cmp(&a.inicio));
        v
    }

    /// Muestra todas las cámaras.
    pub fn camaras(&self) -> Vec<Camara> {
        self.camaras.values().cloned().collect()
    }

    /// Muestra todos los drones
    pub fn drones(&self) -> Vec<Dron> {
        self.drones.values().cloned().collect()
    }

    /// Envía un incidente segun su id.
    pub fn incidente(&self, id: u64) -> Option<Incidente> {
        self.incidentes.get(&id).cloned()
    }

    /// Envía una cámara segun su id.
    pub fn camara(&self, id: u64) -> Option<Camara> {
        self.camaras.get(&id).cloned()
    }

    /// Envía un dron segun su id.
    pub fn dron(&self, id: u64) -> Option<Dron> {
        self.drones.get(&id).cloned()
    }

    /// Limpia todas las cámaras.
    pub fn limpiar_camaras(&mut self) {
        self.camaras.clear();
    }

    pub fn incidente_a_string(&mut self, id_incidente: &u64) -> String {
        let incidente: Option<Incidente> = self.incidente(*id_incidente);
        if let Some(incidente) = incidente {
            return incidente.detalle.to_string();
        }
        "No se encontró el incidente".to_string()
    }
}
