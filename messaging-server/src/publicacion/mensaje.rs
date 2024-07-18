use std::fmt::Debug;

use lib::parseador::mensaje::formatear_payload_debug;

/// Representa un mensaje que se va a publicar en un tópico
#[derive(Clone)]
pub struct PublicacionMensaje {
    pub sid: String,
    pub topico: String,
    pub payload: Vec<u8>,          // El mensaje que se va a enviar
    pub header: Option<Vec<u8>>,   // EL header del mensaje que se va a enviar
    pub replay_to: Option<String>, // Campo que tiene nats
}

impl PublicacionMensaje {
    pub fn new(
        sid: String,
        topico: String,
        payload: Vec<u8>,
        header: Option<Vec<u8>>,
        replay_to: Option<String>,
    ) -> Self {
        Self {
            sid,
            topico,
            payload,
            replay_to,
            header,
        }
    }

    pub fn serializar_msg(&self) -> Vec<u8> {
        // MSG <subject> <sid> [reply-to] <#bytes>␍␊[payload]␍␊
        // HMSG <subject> <sid> [reply-to] <#header bytes> <#total bytes>␍␊[headers]␍␊␍␊[payload]␍␊

        let mut bytes = Vec::new();

        if self.header.is_some() {
            bytes.extend_from_slice(b"HMSG ");
        } else {
            bytes.extend_from_slice(b"MSG ");
        }

        bytes.extend_from_slice(self.topico.as_bytes());
        bytes.extend_from_slice(b" ");

        bytes.extend_from_slice(self.sid.as_bytes());
        bytes.extend_from_slice(b" ");
        if let Some(replay_to) = &self.replay_to {
            bytes.extend_from_slice(replay_to.as_bytes());
            bytes.extend_from_slice(b" ");
        }

        if let Some(header) = &self.header {
            bytes.extend_from_slice(header.len().to_string().as_bytes());
            bytes.extend_from_slice(b" ");
            bytes.extend_from_slice(self.payload.len().to_string().as_bytes());
            bytes.extend_from_slice(b"\r\n");
            bytes.extend_from_slice(header);
            bytes.extend_from_slice(b"\r\n");
        } else {
            bytes.extend_from_slice(self.payload.len().to_string().as_bytes());
            bytes.extend_from_slice(b"\r\n");
        }

        bytes.extend_from_slice(&self.payload);
        bytes.extend_from_slice(b"\r\n");

        bytes
    }
}

impl Debug for PublicacionMensaje {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mensaje")
            .field("topico", &self.topico)
            .field("replay_to", &self.replay_to)
            .field("payload", &formatear_payload_debug(&self.payload))
            .field("header", &self.header)
            .finish()
    }
}
