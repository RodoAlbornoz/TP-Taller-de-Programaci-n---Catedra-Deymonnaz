use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Registro {
    pub nivel: NivelRegistro,
    pub hilo: Option<u64>,
    pub conexion: Option<u64>,
    pub mensaje: String,
}

impl Registro {
    pub fn info(mensaje: String, hilo: Option<u64>, conexion: Option<u64>) -> Registro {
        Registro {
            nivel: NivelRegistro::Informacion,
            hilo,
            conexion,
            mensaje,
        }
    }

    pub fn advertencia(mensaje: String, hilo: Option<u64>, conexion: Option<u64>) -> Registro {
        Registro {
            nivel: NivelRegistro::Advertencia,
            hilo,
            conexion,
            mensaje,
        }
    }

    pub fn error(mensaje: String, hilo: Option<u64>, conexion: Option<u64>) -> Registro {
        Registro {
            nivel: NivelRegistro::Error,
            hilo,
            conexion,
            mensaje,
        }
    }
}

impl Display for Registro {
    /// Formato: `{Nivel} [hilo: {}] [cliente: {}] {Mensaje}`
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(hilo) = self.hilo {
            if let Some(conexion) = self.conexion {
                write!(
                    f,
                    "{} [hilo: {}] [cliente: {}] {}",
                    self.nivel, hilo, conexion, self.mensaje
                )
            } else {
                write!(f, "{} [hilo: {}] {}", self.nivel, hilo, self.mensaje)
            }
        } else if let Some(conexion) = self.conexion {
            write!(f, "{} [cliente: {}] {}", self.nivel, conexion, self.mensaje)
        } else {
            write!(f, "{} {}", self.nivel, self.mensaje)
        }
    }
}

#[derive(Debug, Clone)]
pub enum NivelRegistro {
    Informacion,
    Advertencia,
    Error,
}

impl Display for NivelRegistro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NivelRegistro::Informacion => write!(f, "Info"),
            NivelRegistro::Advertencia => write!(f, "Advertencia"),
            NivelRegistro::Error => write!(f, "Error"),
        }
    }
}
