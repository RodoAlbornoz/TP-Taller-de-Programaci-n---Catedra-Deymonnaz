use crate::{conexion::id::IdConexion, hilo::id::IdHilo};

use super::{id::IdSuscripcion, topico::Topico};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Suscripcion {
    id_hilo: IdHilo,
    id_cliente: IdConexion,
    id: IdSuscripcion,
    topico: Topico,
    id_grupo: Option<IdSuscripcion>,
}

impl Suscripcion {
    pub fn new(
        id_hilo: IdHilo,
        id_cliente: IdConexion,
        topico: Topico,
        id: IdSuscripcion,
        grupo: Option<IdSuscripcion>,
    ) -> Self {
        Self {
            id_hilo,
            id_cliente,
            topico,
            id,
            id_grupo: grupo,
        }
    }

    pub fn topico(&self) -> &Topico {
        &self.topico
    }

    pub fn id(&self) -> &IdSuscripcion {
        &self.id
    }

    pub fn id_hilo(&self) -> &IdHilo {
        &self.id_hilo
    }

    pub fn id_conexion(&self) -> &IdConexion {
        &self.id_cliente
    }

    pub fn id_grupo(&self) -> Option<&IdSuscripcion> {
        self.id_grupo.as_ref()
    }

    pub fn es_grupo(&self) -> bool {
        self.id_grupo.is_some()
    }
}
