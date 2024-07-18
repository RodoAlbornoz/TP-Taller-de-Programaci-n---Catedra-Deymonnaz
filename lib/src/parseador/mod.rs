pub mod mensaje;
pub mod parametros_conectar;
pub mod parametros_info;

mod resultado_linea;

use self::{
    mensaje::Mensaje, parametros_conectar::ParametrosConectar, parametros_info::ParametrosInfo,
    resultado_linea::ResultadoLinea,
};

pub struct Parseador {
    // Bytes que fue acumulando que todavía no se pudieron convertir en ninguna estructura
    bytes_pendientes: Vec<u8>,
    /// Se utiliza internamente para llevar el estado del parseo en diferentes casos
    continuar_en_indice: usize,
    /// La primera linea del mensaje que se está parseando (ejemplo: se encontró un PUB y falta leer el payload)
    actual: Option<ResultadoLinea>,
    /// Los headers del mensaje que se está parseando ahora
    header: Option<Vec<u8>>,
}

/// La responsabilidad del parser es recibir bytes de la conexión y tranformarlos a mensajes
///
/// Acumula los bytes que recibe y cuando tiene suficientes bytes para formar un mensaje
/// lo parsea y lo devuelve
///
/// El proceso de utilizar los bytes que fue recibiendo y convertirlos a mensajes se llama
/// desde la función `proximo_mensaje`, esta te entrega mensajes uno por uno
///
/// El parser se encarga de ir liberando los bytes que ya utilizó y de tener en cuenta los estados intermedios
/// (es decir, si le llega un mensaje que no está completo, lo guarda y espera a que lleguen más bytes)
impl Default for Parseador {
    fn default() -> Self {
        Self::new()
    }
}

impl Parseador {
    pub fn new() -> Self {
        Self {
            bytes_pendientes: Vec::new(),
            continuar_en_indice: 0,
            actual: None,
            header: None,
        }
    }

    /// Agrega bytes al parser
    pub fn agregar_bytes(&mut self, bytes: &[u8]) {
        self.bytes_pendientes.extend_from_slice(bytes);
    }

    /// Devuelve la próxima línea que se encuentra en los bytes que se le pasaron
    /// O `None` si no se encontró ninguna línea (porque no se recibieron suficientes bytes)
    pub fn proxima_linea(&mut self) -> Option<String> {
        let mut last_char_cr = false;

        // Pasar por todos los bytes que tenemos que todavía no procesamos
        // Estamos buscando un salto de linea, este se marca como \r\n
        for i in self.continuar_en_indice..self.bytes_pendientes.len() {
            // Si encontramos un \r, marcamos que el último caracter fue un \r
            if self.bytes_pendientes[i] == b'\r' {
                // La b es para que lo tome como binario
                last_char_cr = true;
            // Si encontramos un \n y el último caracter fue un \r, encontramos un mensaje (o al menos la primera linea)
            } else if last_char_cr && self.bytes_pendientes[i] == b'\n' {
                self.continuar_en_indice = i + 1;

                let result: String =
                    String::from_utf8_lossy(&self.bytes_pendientes[..self.continuar_en_indice])
                        .trim_end() // Elimina los espacios vacios al final
                        .to_string();

                self.resetear_bytes();
                return Some(result);
            } else {
                last_char_cr = false;
            }
        }

        None
    }

    pub fn proximo_mensaje(&mut self) -> Option<Mensaje> {
        // Si actualmente se está parseando un PUB buscamos el payload
        if let Some(ResultadoLinea::Pub(topic, reply_to, bytes_totales)) = &self.actual {
            // No hay suficientes bytes para el payload
            if self.bytes_pendientes.len() < *bytes_totales {
                return None;
            }

            self.continuar_en_indice = *bytes_totales;

            // Si hay suficientes bytes para el payload, devolvemos el mensaje
            let resultado: Option<Mensaje> = Some(Mensaje::Publicar(
                topic.to_string(),
                reply_to.clone(),
                self.bytes_pendientes[..*bytes_totales].to_vec(),
            ));

            self.resetear_todo();

            return resultado;
        }

        // Si actualmente se está parseando un HPUB buscamos el payload
        if let Some(ResultadoLinea::Hpub(topic, reply_to, headers_bytes, bytes_totales)) =
            &self.actual
        {
            let bytes_totales_con_salto_de_linea: usize = *bytes_totales + 2;

            // Si ya habíamos encontrado los headers antes,
            // tenemos todo para buscar el payload y si está completo devolver el mensaje
            if let Some(headers) = &self.header {
                // No hay suficientes bytes para el payload
                if self.bytes_pendientes.len() < bytes_totales_con_salto_de_linea {
                    return None;
                }

                self.continuar_en_indice = bytes_totales_con_salto_de_linea;

                // Si hay suficientes bytes para el payload, devolvemos el mensaje
                let resultado: Option<Mensaje> = Some(Mensaje::PublicarConHeader(
                    topic.to_string(),
                    reply_to.clone(),
                    headers.clone(),
                    self.bytes_pendientes[..*bytes_totales].to_vec(),
                ));

                self.resetear_todo();

                return resultado;
            } else {
                // Si no encontramos los headers antes, buscamos los headers
                if self.bytes_pendientes.len() < *headers_bytes {
                    return None;
                }

                self.header = Some(self.bytes_pendientes[..*headers_bytes].to_vec());
                self.continuar_en_indice = *headers_bytes;
                return self.proximo_mensaje();
            }
        }

        // Si actualmente se está parseando un MSG buscamos el payload
        if let Some(ResultadoLinea::Msg(topic, id_suscripcion, reply_to, bytes_totales)) =
            &self.actual
        {
            // No hay suficientes bytes para el payload
            if self.bytes_pendientes.len() < *bytes_totales {
                return None;
            }

            self.continuar_en_indice = *bytes_totales;

            // Si hay suficientes bytes para el payload, devolvemos el mensaje
            let resultado: Option<Mensaje> = Some(Mensaje::Publicacion(
                topic.to_string(),
                id_suscripcion.to_string(),
                reply_to.clone(),
                self.bytes_pendientes[..*bytes_totales].to_vec(),
            ));

            self.resetear_todo();

            return resultado;
        }

        // Si actualmente se está parseando un HMSG buscamos el payload
        if let Some(ResultadoLinea::Hmsg(topic, sid, reply_to, headers_bytes, bytes_totales)) =
            &self.actual
        {
            let bytes_totales_con_salto_de_linea = *bytes_totales + 2;

            // Si ya habíamos encontrado los headers antes,
            // tenemos todo para buscar el payload y si está completo devolver el mensaje
            if let Some(headers) = &self.header {
                // No hay suficientes bytes para el payload
                if self.bytes_pendientes.len() < bytes_totales_con_salto_de_linea {
                    return None;
                }

                self.continuar_en_indice = bytes_totales_con_salto_de_linea;

                // Si hay suficientes bytes para el payload, devolvemos el mensaje
                let resultado: Option<Mensaje> = Some(Mensaje::PublicacionConHeader(
                    topic.to_string(),
                    sid.to_string(),
                    reply_to.clone(),
                    headers.clone(),
                    self.bytes_pendientes[..*bytes_totales].to_vec(),
                ));

                self.resetear_todo();

                return resultado;
            } else {
                // Si no encontramos los headers antes, buscamos los headers
                if self.bytes_pendientes.len() < *headers_bytes {
                    return None;
                }

                self.header = Some(self.bytes_pendientes[..*headers_bytes].to_vec());
                self.continuar_en_indice = *headers_bytes;
                return self.proximo_mensaje();
            }
        }

        // Si actualmente no se está parseando nada, buscamos la próxima línea
        if self.actual.is_none() {
            let linea: String = self.proxima_linea()?;

            match self.parsear_linea(&linea) {
                ResultadoLinea::Hpub(topico, reply_to, bytes_header, bytes_totales) => {
                    self.actual = Some(ResultadoLinea::Hpub(
                        topico,
                        reply_to,
                        bytes_header,
                        bytes_totales,
                    ));
                }
                ResultadoLinea::MensajeIncorrecto => {
                    return Some(Mensaje::Error("Mensaje incorrecto".to_string()));
                }
                ResultadoLinea::StringVacio => {
                    return self.proximo_mensaje();
                }
                ResultadoLinea::Sub(topico, queue_group, sid) => {
                    return Some(Mensaje::Suscribir(topico, queue_group, sid));
                }
                ResultadoLinea::Unsub(sid, maximos_mensajes) => {
                    return Some(Mensaje::Desuscribir(sid, maximos_mensajes));
                }
                ResultadoLinea::Pub(topico, reply_to, bytes) => {
                    self.actual = Some(ResultadoLinea::Pub(topico, reply_to, bytes));
                    return self.proximo_mensaje();
                }
                ResultadoLinea::Ping => {
                    return Some(Mensaje::Ping());
                }
                ResultadoLinea::Connect(parametros_conectar) => {
                    return Some(Mensaje::Conectar(parametros_conectar));
                }
                ResultadoLinea::Info(parametros) => {
                    return Some(Mensaje::Info(parametros));
                }
                ResultadoLinea::Pong => {
                    return Some(Mensaje::Pong());
                }
                ResultadoLinea::Hmsg(
                    topico,
                    id_suscripcion,
                    reply_to,
                    bytes_header,
                    bytes_contenido,
                ) => {
                    self.actual = Some(ResultadoLinea::Hmsg(
                        topico,
                        id_suscripcion,
                        reply_to,
                        bytes_header,
                        bytes_contenido,
                    ));
                }
                ResultadoLinea::Msg(topico, id_suscripcion, reply_to, bytes_contenido) => {
                    self.actual = Some(ResultadoLinea::Msg(
                        topico,
                        id_suscripcion,
                        reply_to,
                        bytes_contenido,
                    ));
                    return self.proximo_mensaje();
                }
                _ => {}
            }
        }
        None
    }

    /// Se mira la primera palabra de la linea y se devuelve la linea de
    /// resultado segun el tipo de mensaje
    fn parsear_linea(&self, linea: &str) -> ResultadoLinea {
        let palabras: Vec<String> = linea
            .split(' ')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let palabra_vacia: String = "".to_string();
        let primera_palabra: String = palabras.first().unwrap_or(&palabra_vacia).to_lowercase();

        if primera_palabra.eq("pub") {
            return Self::linea_pub(&palabras[1..]);
        }

        if primera_palabra.eq("hpub") {
            return Self::linea_hpub(&palabras[1..]);
        }

        if primera_palabra.eq("sub") {
            return Self::linea_sub(&palabras[1..]);
        }

        if primera_palabra.eq("unsub") {
            return Self::linea_unsub(&palabras[1..]);
        }

        if primera_palabra.eq("") {
            return ResultadoLinea::StringVacio;
        }

        if primera_palabra.eq("ping") {
            return ResultadoLinea::Ping;
        }

        if primera_palabra.eq("pong") {
            return ResultadoLinea::Pong;
        }

        if primera_palabra.eq("+ok") {
            return ResultadoLinea::Ok;
        }

        if primera_palabra.eq("-err") {
            return ResultadoLinea::Err;
        }

        if primera_palabra.eq("connect") {
            if let Ok(parametros_conectar) =
                ParametrosConectar::desde_json(&palabras[1..].join(" "))
            {
                return ResultadoLinea::Connect(parametros_conectar);
            }
        }

        if primera_palabra.eq("info") {
            if let Ok(parametros_info) = ParametrosInfo::desde_json(&palabras[1..].join(" ")) {
                return ResultadoLinea::Info(parametros_info);
            }
        }

        if primera_palabra.eq("msg") {
            return Self::linea_msg(&palabras[1..]);
        }

        if primera_palabra.eq("hmsg") {
            return Self::linea_hmsg(&palabras[1..]);
        }

        ResultadoLinea::MensajeIncorrecto
    }

    fn linea_pub(palabras: &[String]) -> ResultadoLinea {
        // Buscamos si es de 2 o 3 para saber si tiene reply_to
        if palabras.len() == 2 {
            let bytes: usize = match palabras[1].parse() {
                Ok(b) => b,
                Err(_) => return ResultadoLinea::MensajeIncorrecto,
            };

            return ResultadoLinea::Pub(palabras[0].to_string(), None, bytes);
        }

        if palabras.len() == 3 {
            let bytes: usize = match palabras[2].parse() {
                Ok(b) => b,
                Err(_) => return ResultadoLinea::MensajeIncorrecto,
            };

            return ResultadoLinea::Pub(
                palabras[0].to_string(),
                Some(palabras[1].to_string()),
                bytes,
            );
        }

        ResultadoLinea::MensajeIncorrecto
    }

    fn linea_hpub(palabras: &[String]) -> ResultadoLinea {
        // Buscamos si es de 3 o 4 para saber si tiene reply_to
        if palabras.len() == 3 {
            let bytes: usize = match palabras[1].parse() {
                Ok(b) => b,
                Err(_) => return ResultadoLinea::MensajeIncorrecto,
            };
            let headers_bytes: usize = match palabras[2].parse() {
                Ok(b) => b,
                Err(_) => return ResultadoLinea::MensajeIncorrecto,
            };

            return ResultadoLinea::Hpub(palabras[0].to_string(), None, headers_bytes, bytes);
        }

        if palabras.len() == 4 {
            let bytes: usize = match palabras[2].parse() {
                Ok(b) => b,
                Err(_) => return ResultadoLinea::MensajeIncorrecto,
            };
            let headers_bytes: usize = match palabras[3].parse() {
                Ok(b) => b,
                Err(_) => return ResultadoLinea::MensajeIncorrecto,
            };

            return ResultadoLinea::Hpub(
                palabras[0].to_string(),
                Some(palabras[1].to_string()),
                headers_bytes,
                bytes,
            );
        }

        ResultadoLinea::MensajeIncorrecto
    }

    fn linea_sub(palabras: &[String]) -> ResultadoLinea {
        // Buscamos si es de 2 o 3 para saber si tiene queue_group
        if palabras.len() == 2 {
            let topico: &String = &palabras[0];
            let sid: &String = &palabras[1];
            return ResultadoLinea::Sub(topico.to_string(), None, sid.to_string());
        }

        if palabras.len() == 3 {
            let topico: &String = &palabras[0];
            let queue_group: &String = &palabras[1];
            let sid: &String = &palabras[2];

            return ResultadoLinea::Sub(
                topico.to_string(),
                Some(queue_group.to_string()),
                sid.to_string(),
            );
        }

        ResultadoLinea::MensajeIncorrecto
    }

    fn linea_unsub(palabras: &[String]) -> ResultadoLinea {
        if palabras.len() != 1 {
            return ResultadoLinea::MensajeIncorrecto;
        }

        let sid: &String = &palabras[0];
        let maximos_mensajes: Option<u64> = palabras.get(2).map(|s| s.parse::<u64>().unwrap());

        ResultadoLinea::Unsub(sid.to_string(), maximos_mensajes)
    }

    fn linea_msg(palabras: &[String]) -> ResultadoLinea {
        // Buscamos si es de 3 o 4 para saber si tiene reply_to
        if palabras.len() == 3 {
            let bytes: usize = match palabras[2].parse() {
                Ok(b) => b,
                Err(_) => return ResultadoLinea::MensajeIncorrecto,
            };

            return ResultadoLinea::Msg(
                palabras[0].to_string(),
                palabras[1].to_string(),
                None,
                bytes,
            );
        }

        if palabras.len() == 4 {
            let bytes: usize = match palabras[3].parse() {
                Ok(b) => b,
                Err(_) => return ResultadoLinea::MensajeIncorrecto,
            };

            return ResultadoLinea::Msg(
                palabras[0].to_string(),
                palabras[1].to_string(),
                Some(palabras[2].to_string()),
                bytes,
            );
        }

        ResultadoLinea::MensajeIncorrecto
    }

    fn linea_hmsg(palabras: &[String]) -> ResultadoLinea {
        // Buscamos si es de 4 o 5 para saber si tiene reply_to
        if palabras.len() == 4 {
            let bytes: usize = match palabras[3].parse() {
                Ok(b) => b,
                Err(_) => return ResultadoLinea::MensajeIncorrecto,
            };

            let headers_bytes: usize = match palabras[2].parse() {
                Ok(b) => b,
                Err(_) => return ResultadoLinea::MensajeIncorrecto,
            };

            return ResultadoLinea::Hmsg(
                palabras[0].to_string(),
                palabras[1].to_string(),
                None,
                headers_bytes,
                bytes,
            );
        }

        if palabras.len() == 5 {
            let bytes: usize = match palabras[4].parse() {
                Ok(b) => b,
                Err(_) => return ResultadoLinea::MensajeIncorrecto,
            };

            let headers_bytes: usize = match palabras[3].parse() {
                Ok(b) => b,
                Err(_) => return ResultadoLinea::MensajeIncorrecto,
            };

            return ResultadoLinea::Hmsg(
                palabras[0].to_string(),
                palabras[1].to_string(),
                Some(palabras[2].to_string()),
                headers_bytes,
                bytes,
            );
        }

        ResultadoLinea::MensajeIncorrecto
    }

    /// Libera los bytes de la parte del mensaje que ya se parseó
    ///
    /// Por ejemplo:
    /// - La primera linea de cualquier mensaje
    /// - El payload
    /// - Los headers
    fn resetear_bytes(&mut self) {
        self.bytes_pendientes.drain(..self.continuar_en_indice);
        self.continuar_en_indice = 0;
    }

    fn resetear_todo(&mut self) {
        self.resetear_bytes();
        self.actual = None;
        self.header = None;
    }
}

#[cfg(test)]
mod tests {
    use crate::parseador::resultado_linea::ResultadoLinea;

    #[test]
    fn linea_sub() {
        let parser = super::Parseador::new();
        let resultado = parser.parsear_linea("sub subject sid");
        assert_eq!(
            resultado,
            ResultadoLinea::Sub("subject".to_string(), None, "sid".to_string())
        );

        let resultado = parser.parsear_linea("sub subject queue_group sid");
        assert_eq!(
            resultado,
            ResultadoLinea::Sub(
                "subject".to_string(),
                Some("queue_group".to_string()),
                "sid".to_string()
            )
        );

        let resultado = parser.parsear_linea("sub");
        assert_eq!(resultado, ResultadoLinea::MensajeIncorrecto);
    }
}
