use std::io;

use lib::{
    csv::csv_parsear_linea,
    serializables::{error::DeserializationError, guardar::cargar_serializable, Serializable},
};

#[derive(Debug, Clone)]
pub struct Cuenta {
    pub id: u64,
    pub user: String,
    pub pass: String,
}

impl Cuenta {
    pub fn matches(&self, user: &str, pass: &str) -> bool {
        self.user == user && self.pass == pass
    }

    pub fn cargar(ruta_archivo: &str) -> Result<Vec<Cuenta>, io::Error> {
        cargar_serializable(ruta_archivo)
    }
}

impl Serializable for Cuenta {
    fn serializar(&self) -> Vec<u8> {
        format!("{},{},{}", self.id, self.user, self.pass).into()
    }

    fn deserializar(data: &[u8]) -> Result<Self, DeserializationError>
    where
        Self: Sized,
    {
        let linea =
            String::from_utf8(data.to_vec()).map_err(|_| DeserializationError::InvalidData)?;
        let mut parametros = csv_parsear_linea(linea.as_str()).into_iter();

        let id = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let user = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?;
        let pass = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?;

        Ok(Cuenta {
            id,
            user: user.to_string(),
            pass: pass.to_string(),
        })
    }
}
