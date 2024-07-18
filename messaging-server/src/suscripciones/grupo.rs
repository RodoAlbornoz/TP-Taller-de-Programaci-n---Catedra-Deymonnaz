use std::collections::HashSet;

use rand::{thread_rng, Rng};

use super::{id::IdSuscripcion, suscripcion::Suscripcion, topico::Topico};

pub struct Grupo {
    id: IdSuscripcion,
    topico: Topico,
    suscripciones: HashSet<Suscripcion>,
}

impl Grupo {
    pub fn new(id: IdSuscripcion, topico: Topico) -> Self {
        Self {
            id,
            topico,
            suscripciones: HashSet::new(),
        }
    }

    pub fn id(&self) -> &IdSuscripcion {
        &self.id
    }

    pub fn topico(&self) -> &Topico {
        &self.topico
    }

    pub fn suscribir(&mut self, suscripcion: Suscripcion) {
        self.suscripciones.insert(suscripcion);
    }

    pub fn desuscribir(&mut self, suscripcion: &Suscripcion) {
        self.suscripciones.remove(suscripcion);
    }

    pub fn suscripcion_random(&self) -> Option<&Suscripcion> {
        let index = thread_rng().gen_range(0..self.suscripciones.len());
        return self.suscripciones.iter().nth(index);
    }
}
