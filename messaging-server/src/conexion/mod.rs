pub mod id;
pub mod respuesta;
pub mod tick_contexto;

use lib::parseador::mensaje::{formatear_mensaje_debug, formatear_payload_debug};
use lib::parseador::parametros_info::ParametrosInfo;
use lib::parseador::Parseador;
use lib::{parseador::mensaje::Mensaje, stream::Stream};
use std::sync::Arc;
use std::{fmt::Debug, io};

use chrono::{DateTime, Local};

use crate::cuenta::Cuenta;
use crate::{
    publicacion::{mensaje::PublicacionMensaje, Publicacion},
    registrador::Registrador,
    suscripciones::{suscripcion::Suscripcion, topico::Topico},
};

use self::{id::IdConexion, respuesta::Respuesta, tick_contexto::TickContexto};
pub struct Conexion {
    /// El identificador de la conexión. Global y único
    id: IdConexion,
    /// El stream de la conexión
    stream: Box<dyn Stream>,
    /// Registrador de eventos
    registrador: Registrador,
    /// El parser se encarga de leer los bytes y generar mensajes
    parser: Parseador,
    /// Tiempo del ultimo PING
    tiempo_ultimo_ping: DateTime<Local>,

    pub desconectado: bool,

    /// Indica si la conexión está autenticada.
    /// Es decir, si ya se envió un mensaje de conexión (`CONNECT {...}`)
    pub autenticado: bool,

    /// Cuentas de usuario
    pub cuentas: Option<Arc<Vec<Cuenta>>>,
}

impl Conexion {
    pub fn new(
        id: IdConexion,
        stream: Box<dyn Stream>,
        registrador: Registrador,
        cuentas: Option<Arc<Vec<Cuenta>>>,
    ) -> Self {
        let mut con = Self {
            id,
            stream,
            parser: Parseador::new(),
            registrador,
            tiempo_ultimo_ping: Local::now(),
            desconectado: false,
            autenticado: false,
            cuentas,
        };

        con.enviar_info();

        con
    }

    pub fn tick(&mut self, salida: &mut TickContexto) {
        if self.desconectado {
            return;
        }
        // Si hace falta enviar un PING o no
        if self.enviar_ping() {
            _ = self.escribir_bytes(b"PING\r\n");
        }

        // Lee los bytes del stream y los envía al parser
        self.leer_bytes();

        // Lee mensaje y actua en consecuencia
        self.leer_mensajes(salida);
    }

    /// Chequea si pasaron 20 segundos desde el ultimo PING enviado
    fn enviar_ping(&mut self) -> bool {
        let tiempo_actual = Local::now();
        let duracion_ultimo_ping = tiempo_actual.signed_duration_since(self.tiempo_ultimo_ping);

        if duracion_ultimo_ping.num_seconds() >= 20 {
            self.tiempo_ultimo_ping = tiempo_actual;
            true
        } else {
            false
        }
    }

    /// Este método lo envia el Hilo cuando recibe un mensaje
    pub fn escribir_publicacion_mensaje(&mut self, mensaje: &PublicacionMensaje) {
        self.registrador
            .info(&format!("MSG: {:?}", mensaje), Some(self.id));

        if self.escribir_bytes(&mensaje.serializar_msg()).is_err() {
            self.registrador
                .advertencia("Error al enviar mensaje", Some(self.id));
        }
    }

    /// Lee los bytes del stream y los envía al parser
    pub fn leer_bytes(&mut self) {
        let mut buffer = [0; 1024]; // 1kb
                                    // 1. Leer una vez
        match self.stream.read(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    self.desconectado = true;
                    return;
                }

                // 2. Enviar bytes a parser y leer nuevos mensajes generados
                self.parser.agregar_bytes(&buffer[..n]);
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // No hay datos para leer (no hay que hacer nada acá)
            }
            Err(e) => {
                self.registrador
                    .error(&format!("Error al leer del stream {}", e), Some(self.id));
                self.registrador.error("Error al leer bytes", Some(self.id));

                self.desconectado = true;
            }
        }
    }

    /// Escribir al stream
    pub fn escribir_bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        if let Err(e) = self.stream.write_all(bytes) {
            self.registrador
                .advertencia(&format!("Error al escribir al stream {}", e), Some(self.id));
            self.desconectado = true;
            return Err(e);
        }

        Ok(())
    }

    pub fn escribir_respuesta(&mut self, respuesta: &Respuesta) {
        let bytes = &respuesta.serializar();
        if self.escribir_bytes(bytes).is_err() {
            self.registrador
                .error("Error al enviar respuesta", Some(self.id));
        }
    }

    pub fn escribir_ok(&mut self, msg: Option<String>) {
        self.escribir_respuesta(&Respuesta::Ok(msg));
    }

    pub fn escribir_err(&mut self, msg: Option<String>) {
        self.escribir_respuesta(&Respuesta::Err(msg));
    }

    pub fn enviar_info(&mut self) {
        let require_auth = self.cuentas.is_some();
        self.escribir_respuesta(&Respuesta::Info(ParametrosInfo {
            requiere_auth: Some(require_auth),
        }));
    }

    pub fn leer_mensajes(&mut self, contexto: &mut TickContexto) {
        while let Some(mensaje) = self.parser.proximo_mensaje() {
            self.registrador.info(
                &format!("Mensaje recibido: {:?}", formatear_mensaje_debug(&mensaje)),
                Some(self.id),
            );

            if !self.autenticado {
                match mensaje {
                    Mensaje::Conectar(parametros) => {
                        if let Some(cuentas) = &self.cuentas {
                            for cuenta in cuentas.iter() {
                                if cuenta.matches(&parametros.user_str(), &parametros.pass_str()) {
                                    self.registrador.info(
                                        &format!("Usuario autenticado: {}", cuenta.user),
                                        Some(self.id),
                                    );

                                    self.autenticado = true;
                                    self.escribir_respuesta(&Respuesta::Ok(Some(
                                        "connect".to_string(),
                                    )));
                                    return;
                                }
                            }

                            self.escribir_err(Some("Usuario o contraseña incorrectos".to_string()));
                            self.desconectado = true;
                            return;
                        }

                        self.autenticado = true;
                        self.escribir_respuesta(&Respuesta::Ok(Some("connect".to_string())));
                    }
                    _ => {
                        self.escribir_err(Some(
                            "Primero debe enviar un mensaje de conexión".to_string(),
                        ));
                        self.desconectado = true;
                        return;
                    }
                }
                continue;
            }

            // proximo mensaje va a leer los bytes nuevos y devuelve si es una accion valida
            match mensaje {
                Mensaje::Publicar(subject, replay_to, payload) => {
                    self.registrador.info(
                        &format!(
                            "Publicación: {:?} {:?} {:?}",
                            subject,
                            replay_to,
                            formatear_payload_debug(&payload)
                        ),
                        Some(self.id),
                    );

                    contexto.publicar(Publicacion::new(subject, payload, None, replay_to));
                    self.escribir_ok(Some("pub".to_string()));
                }
                Mensaje::PublicarConHeader(subject, replay_to, headers, payload) => {
                    self.registrador.info(
                        &format!(
                            "Publicación con header: {:?} {:?} {:?} {:?}",
                            subject,
                            headers,
                            replay_to,
                            formatear_payload_debug(&payload)
                        ),
                        Some(self.id),
                    );

                    contexto.publicar(Publicacion::new(subject, payload, Some(headers), replay_to));
                    self.escribir_ok(Some("hpub".to_string()));
                }
                Mensaje::Suscribir(topico, grupo, id) => match Topico::new(topico) {
                    Ok(topico) => {
                        contexto.suscribir(Suscripcion::new(
                            contexto.id_hilo,
                            self.id,
                            topico,
                            id,
                            grupo,
                        ));
                        self.escribir_ok(Some("sub".to_string()));
                    }
                    Err(_) => {
                        self.escribir_err(Some("Tópico de subscripción incorrecto".to_string()));
                    }
                },
                Mensaje::Desuscribir(id, _max_msgs) => {
                    contexto.desuscribir(id);
                    self.escribir_ok(Some("unsub".to_string()));
                }
                Mensaje::Error(msg) => {
                    // self.respuestas.push(Respuesta::Err(msg));
                    self.escribir_err(Some(msg));
                }
                Mensaje::Conectar(_) => {
                    self.escribir_err(Some("Ya se recibió un mensaje de conexión".to_string()));
                }
                Mensaje::Ping() => {
                    self.escribir_respuesta(&Respuesta::Pong());
                }
                _ => {
                    self.escribir_respuesta(&Respuesta::Err(Some(
                        "Mensaje no reconocido".to_string(),
                    )));
                }
            }
        }
    }

    pub fn esta_conectado(&self) -> bool {
        !self.desconectado
    }
}

impl Debug for Conexion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Conexion")
            .field("id", &self.id)
            .field("desconectado", &self.desconectado)
            .field("autenticado", &self.autenticado)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use lib::{serializables::deserializar_vec, stream::mock_handler::MockHandler};

    use crate::registrador::Registrador;

    use super::{tick_contexto::TickContexto, Conexion};

    #[test]
    fn probar_info() {
        // El MockStream simula ser el stream del cliente, el control permite leer y escribir al stream
        let (mut control, stream) = MockHandler::new();
        let registrador = Registrador::new();

        // Conexion representa el cliente del lado del servidor
        Conexion::new(1, Box::new(stream), registrador, None);

        assert!(control
            .intentar_recibir_string()
            .unwrap()
            .to_uppercase()
            .starts_with("INFO"));
    }

    #[test]
    fn probar_autenticacion_sin_cuenta() {
        let (mut mock, stream) = MockHandler::new();
        let registrador = Registrador::new();

        let mut con = Conexion::new(1, Box::new(stream), registrador, None);

        mock.escribir_bytes(b"CONNECT {}\r\n");

        let mut contexto = TickContexto::new(0, 1);
        con.tick(&mut contexto);

        assert!(con.autenticado);
    }

    #[test]
    fn probar_autenticacion_con_cuenta() {
        let (mut mock, stream) = MockHandler::new();
        let registrador = Registrador::new();

        let cuentas = deserializar_vec("1,admin,1234".as_bytes()).unwrap();

        let mut con = Conexion::new(1, Box::new(stream), registrador, Some(Arc::new(cuentas)));

        mock.escribir_bytes(b"CONNECT {\"user\": \"admin\", \"pass\": \"1234\"}\r\n");

        let mut contexto = TickContexto::new(0, 1);
        con.tick(&mut contexto);

        assert!(con.autenticado);
    }

    #[test]
    fn probar_suscripcion() {
        let (mut mock, stream) = MockHandler::new();
        let registrador = Registrador::new();

        let mut con = Conexion::new(1, Box::new(stream), registrador, None);
        mock.escribir_bytes(b"CONNECT {\"user\": \"admin\", \"pass\": \"admin\"}\r\n");

        let mut contexto = TickContexto::new(0, 1);
        con.tick(&mut contexto);

        mock.escribir_bytes(b"SUB x 1\r\n");

        let mut contexto = TickContexto::new(0, 1);
        con.tick(&mut contexto);

        assert_eq!(contexto.suscripciones.len(), 1);
        assert_eq!(contexto.suscripciones[0].id(), "1");
        assert_eq!(contexto.suscripciones[0].topico().a_texto(), "x");
    }

    #[test]
    fn probar_publicar() {
        let (mut mock, stream) = MockHandler::new();
        let registrador = Registrador::new();

        let mut con = Conexion::new(1, Box::new(stream), registrador, None);
        mock.escribir_bytes(b"CONNECT {\"user\": \"admin\", \"pass\": \"admin\"}\r\n");

        let mut contexto = TickContexto::new(0, 1);
        con.tick(&mut contexto);

        mock.escribir_bytes(b"PUB x 4\r\nhola\r\n");

        let mut contexto = TickContexto::new(0, 1);
        con.tick(&mut contexto);

        println!("{:?}", contexto);

        assert_eq!(contexto.publicaciones.len(), 1);
        assert_eq!(contexto.publicaciones[0].topico, "x");
        assert_eq!(contexto.publicaciones[0].payload, b"hola");
    }

    #[test]
    fn probar_desuscripcion() {
        let (mut mock, stream) = MockHandler::new();
        let registrador = Registrador::new();

        let mut con = Conexion::new(1, Box::new(stream), registrador, None);
        mock.escribir_bytes(b"CONNECT {\"user\": \"admin\", \"pass\": \"admin\"}\r\n");

        let mut contexto = TickContexto::new(0, 1);
        con.tick(&mut contexto);

        mock.escribir_bytes(b"UNSUB 1\r\n");

        let mut contexto = TickContexto::new(0, 1);
        con.tick(&mut contexto);

        assert_eq!(contexto.desuscripciones.len(), 1);
        assert_eq!(contexto.desuscripciones[0], "1");
    }
}
