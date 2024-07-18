use lib::parseador::parametros_info::ParametrosInfo;

#[derive(Debug)]
pub enum Respuesta {
    Err(Option<String>),
    Ok(Option<String>),
    Pong(),
    Info(ParametrosInfo),
}

impl Respuesta {
    pub fn serializar(&self) -> Vec<u8> {
        match self {
            Respuesta::Err(error) => {
                let mut bytes = Vec::new();
                bytes.extend_from_slice(b"-ERR ");
                if let Some(error) = error {
                    bytes.extend_from_slice(error.as_bytes());
                }
                bytes.extend_from_slice(b"\r\n");
                bytes
            }
            Respuesta::Ok(msg) => {
                let mut bytes = Vec::new();
                bytes.extend_from_slice(b"+OK");
                if let Some(msg) = msg {
                    bytes.extend_from_slice(b" ");
                    bytes.extend_from_slice(msg.as_bytes());
                }
                bytes.extend_from_slice(b"\r\n");
                bytes
            }
            Respuesta::Pong() => {
                let mut bytes = Vec::new();
                bytes.extend_from_slice(b"PONG");
                bytes.extend_from_slice(b"\r\n");
                bytes
            }
            Respuesta::Info(info) => {
                let json = info.hacia_json().unwrap_or("{}".to_string());

                let mut bytes = Vec::new();
                bytes.extend_from_slice(format!("INFO {}\r\n", json).as_bytes());
                bytes.extend_from_slice(b"\r\n");
                bytes
            }
        }
    }
}
