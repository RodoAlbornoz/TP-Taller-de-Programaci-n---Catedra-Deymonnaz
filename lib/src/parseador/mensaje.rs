use super::{parametros_conectar::ParametrosConectar, parametros_info::ParametrosInfo};

#[derive(Debug)]
pub enum Mensaje {
    // 'topico', 'replay_to' payload
    Publicar(String, Option<String>, Vec<u8>),
    // 'topico', 'replay_to' headers, payload
    PublicarConHeader(String, Option<String>, Vec<u8>, Vec<u8>),
    // 'topico', 'queue group', 'id
    Suscribir(String, Option<String>, String),
    // 'id_suscripcion', 'maximos_mensajes'
    Desuscribir(String, Option<u64>),
    // Mensaje de error (Cuando no se pudo parsear el mensaje)
    Error(String),
    // Mensaje para generar la conexión
    Conectar(ParametrosConectar),
    // Mensaje para preservar la conexión
    Ping(),
    // Mensaje para preservar la conexión
    Pong(),
    //
    Info(ParametrosInfo),
    // MSG <subject> <sid> [reply-to] payload
    Publicacion(String, String, Option<String>, Vec<u8>),
    // HMSG <subject> <sid> [reply-to] headers payload
    PublicacionConHeader(String, String, Option<String>, Vec<u8>, Vec<u8>),
}

pub fn formatear_payload_debug(payload: &[u8]) -> String {
    let mut str: String = String::from_utf8_lossy(payload).to_string();

    if str.len() > 100 {
        str.truncate(100);
        str.push_str("...");
    }

    str
}

pub fn formatear_mensaje_debug(mensaje: &Mensaje) -> String {
    if let Mensaje::Publicar(topico, reply_to, payload) = mensaje {
        return format!(
            "MensajePublicar({:?}, {:?}, {:?})",
            topico,
            reply_to,
            formatear_payload_debug(payload)
        );
    } else if let Mensaje::PublicarConHeader(topico, reply_to, headers, payload) = mensaje {
        return format!(
            "MensajePublicarConHeader({:?}, {:?}, {:?}, {:?})",
            topico,
            reply_to,
            formatear_payload_debug(headers),
            formatear_payload_debug(payload)
        );
    }

    format!("{:?}", mensaje)
}
