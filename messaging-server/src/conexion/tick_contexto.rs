use crate::{
    hilo::id::IdHilo,
    publicacion::Publicacion,
    suscripciones::{id::IdSuscripcion, suscripcion::Suscripcion},
};

use super::id::IdConexion;

#[derive(Debug)]
pub struct TickContexto {
    pub suscripciones: Vec<Suscripcion>,
    pub desuscripciones: Vec<IdSuscripcion>,
    pub publicaciones: Vec<Publicacion>,
    pub id_hilo: IdHilo,
    pub id_conexion: IdConexion,
}

impl TickContexto {
    pub fn new(id_hilo: IdHilo, id_conexion: IdConexion) -> Self {
        Self {
            suscripciones: Vec::new(),
            desuscripciones: Vec::new(),
            publicaciones: Vec::new(),
            id_hilo,
            id_conexion,
        }
    }

    pub fn suscribir(&mut self, suscripcion: Suscripcion) {
        self.suscripciones.push(suscripcion);
    }

    pub fn desuscribir(&mut self, id_suscripcion: IdSuscripcion) {
        self.desuscripciones.push(id_suscripcion);
    }

    pub fn publicar(&mut self, publicacion: Publicacion) {
        self.publicaciones.push(publicacion);
    }
}
