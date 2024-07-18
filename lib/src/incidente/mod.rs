use std::vec::IntoIter;

use crate::{
    coordenadas::Coordenadas,
    csv::{self, csv_encodear_linea},
    serializables::{error::DeserializationError, Serializable},
};

#[derive(Clone, Debug, PartialEq)]
pub struct Incidente {
    pub id: u64,
    pub detalle: String,
    pub latitud: f64,
    pub longitud: f64,
    pub inicio: u64,
}

impl Incidente {
    pub fn new(id: u64, detalle: String, latitud: f64, longitud: f64, inicio: u64) -> Self {
        Incidente {
            id,
            detalle,
            latitud,
            longitud,
            inicio,
        }
    }

    pub fn posicion(&self) -> Coordenadas {
        Coordenadas::a_partir_de_latitud_longitud(self.latitud, self.longitud)
    }
}

impl Serializable for Incidente {
    /// Para serializar un incidente, se toma un vector con su id, detalle, latitud,
    /// longitud e inicio, y se convierte a una linea de un archivo csv, separando
    /// cada parámetro por comas
    fn serializar(&self) -> Vec<u8> {
        let mut parametros: Vec<String> = Vec::new();
        parametros.push(format!("{}", self.id));
        parametros.push(self.detalle.clone());
        parametros.push(format!("{}", self.latitud));
        parametros.push(format!("{}", self.longitud));
        parametros.push(format!("{}", self.inicio));
        csv_encodear_linea(&parametros).into_bytes()
    }

    /// Para deserializar un incidente, se toman los bytes del incidente, se convierte
    /// a un vector de strings (Linea de un csv), se parsea la linea para tomar cada
    /// parámetro de la linea, y a partir de ellos se crea un incidente
    fn deserializar(datos: &[u8]) -> Result<Self, DeserializationError>
    where
        Self: Sized,
    {
        let linea: String =
            String::from_utf8(datos.to_vec()).map_err(|_| DeserializationError::InvalidData)?;

        let mut parametros: IntoIter<String> = csv::csv_parsear_linea(linea.as_str()).into_iter();

        let id: u64 = parametros
            .next()
            .ok_or(DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let detalle: String = parametros.next().ok_or(DeserializationError::InvalidData)?;

        let latitud: f64 = parametros
            .next()
            .ok_or(DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let longitud: f64 = parametros
            .next()
            .ok_or(DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let inicio: u64 = parametros
            .next()
            .ok_or(DeserializationError::InvalidData)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        Ok(Incidente {
            id,
            detalle,
            latitud,
            longitud,
            inicio,
        })
    }
}
