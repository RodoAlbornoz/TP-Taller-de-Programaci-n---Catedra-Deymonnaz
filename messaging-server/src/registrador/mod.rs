use std::sync::mpsc::{channel, Sender};

use crate::hilo::id::IdHilo;

use self::{hilo::hilo_registrador, registro::Registro};

mod hilo;
pub mod registro;

pub struct Registrador {
    emisor: Sender<Registro>,
    hilo: Option<IdHilo>,
}

impl Default for Registrador {
    fn default() -> Self {
        Self::new()
    }
}

impl Registrador {
    pub fn new() -> Self {
        let (emisor, receptor) = channel();
        hilo_registrador(receptor);

        Registrador { emisor, hilo: None }
    }

    /// Establece el valor por defecto del hilo
    pub fn establecer_hilo(&mut self, hilo: IdHilo) {
        self.hilo = Some(hilo);
    }

    /// Registra un evento
    pub fn registrar(&self, registro: Registro) {
        let _ = self.emisor.send(registro);
    }

    /// Registra un evento de información utilizando el hilo por defecto
    pub fn info(&self, mensaje: &str, conexion: Option<u64>) {
        self.registrar(Registro::info(mensaje.to_owned(), self.hilo, conexion));
    }

    /// Registra un evento de advertencia utilizando el hilo por defecto
    pub fn advertencia(&self, mensaje: &str, conexion: Option<u64>) {
        self.registrar(Registro::advertencia(
            mensaje.to_owned(),
            self.hilo,
            conexion,
        ));
    }

    /// Registra un evento de error utilizando el hilo por defecto
    pub fn error(&self, mensaje: &str, conexion: Option<u64>) {
        self.registrar(Registro::error(mensaje.to_owned(), self.hilo, conexion));
    }
}

impl Clone for Registrador {
    /// El clone está implementado para que se pueda clonar el Registrador
    /// y estar seguro que el emisor es único.
    fn clone(&self) -> Self {
        Registrador {
            emisor: self.emisor.clone(),
            // El hilo no se clona. Esto es para evitar errores de consistencia
            // podría pasar que se clone entre hilos e imprima el hilo incorrecto
            hilo: None,
        }
    }
}
