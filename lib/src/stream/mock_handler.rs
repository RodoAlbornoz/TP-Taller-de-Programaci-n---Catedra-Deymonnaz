use std::sync::mpsc::{Receiver, Sender};

use crate::parseador::{mensaje::Mensaje, Parseador};

use super::mock::MockStream;

pub struct MockHandler {
    pub enviar: Sender<Vec<u8>>,
    pub recibir: Receiver<Vec<u8>>,
    pub parseador: Parseador,
}

impl MockHandler {
    pub fn new() -> (Self, MockStream) {
        let (enviar, rx) = std::sync::mpsc::channel();
        let (tx, recibir) = std::sync::mpsc::channel();

        let parseador = Parseador::new();

        (
            Self {
                recibir,
                enviar,
                parseador,
            },
            MockStream::new(tx, rx),
        )
    }

    pub fn escribir_bytes(&mut self, bytes: &[u8]) {
        self.enviar.send(bytes.to_vec()).unwrap();
    }

    pub fn proximo_mensaje(&mut self) -> Option<Mensaje> {
        while let Ok(bytes) = self.recibir.try_recv() {
            self.parseador.agregar_bytes(&bytes)
        }

        self.parseador.proximo_mensaje()
    }

    pub fn intentar_recibir_string(&mut self) -> Option<String> {
        let mut bytes_todos = Vec::new();

        while let Ok(bytes) = self.recibir.try_recv() {
            bytes_todos.extend_from_slice(&bytes)
        }

        if bytes_todos.is_empty() {
            return None;
        }

        Some(String::from_utf8_lossy(&bytes_todos).to_string())
    }
}
