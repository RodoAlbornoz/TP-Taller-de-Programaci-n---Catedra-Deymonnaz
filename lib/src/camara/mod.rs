use std::{collections::HashSet, vec::IntoIter};

use crate::{
    coordenadas::Coordenadas,
    csv::{csv_encodear_linea, csv_parsear_linea},
    serializables::{error::DeserializationError, Serializable},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Camara {
    pub id: u64,
    pub latitud: f64,
    pub longitud: f64,
    pub rango: f64,
    /// Incidentes atendidos por cada camara.
    pub incidentes_primarios: HashSet<u64>,
    /// Incidentes que atiende cada cámara por ser lindante
    pub incidentes_secundarios: HashSet<u64>,
}

impl Camara {
    pub fn new(id: u64, latitud: f64, longitud: f64, rango: f64) -> Self {
        Camara {
            id,
            latitud,
            longitud,
            rango,
            incidentes_primarios: HashSet::new(),
            incidentes_secundarios: HashSet::new(),
        }
    }

    pub fn activa(&self) -> bool {
        !self.incidentes_primarios.is_empty() || !self.incidentes_secundarios.is_empty()
    }

    pub fn posicion(&self) -> Coordenadas {
        Coordenadas::a_partir_de_latitud_longitud(self.latitud, self.longitud)
    }
}

impl Serializable for Camara {
    fn serializar(&self) -> Vec<u8> {
        let mut parametros: Vec<String> = Vec::new();
        parametros.push(format!("{}", self.id));
        parametros.push(format!("{}", self.latitud));
        parametros.push(format!("{}", self.longitud));
        parametros.push(format!("{}", self.rango));
        parametros.push(serializar_vector_incidentes(&self.incidentes_primarios).to_string());
        parametros.push(serializar_vector_incidentes(&self.incidentes_secundarios).to_string());
        csv_encodear_linea(&parametros).into_bytes()
    }

    fn deserializar(data: &[u8]) -> Result<Self, DeserializationError> {
        let linea: String =
            String::from_utf8(data.to_vec()).map_err(|_| DeserializationError::InvalidData)?;
        let mut parametros: IntoIter<String> = csv_parsear_linea(linea.as_str()).into_iter();

        let id: u64 = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let latitud: f64 = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let longitud: f64 = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let rango: f64 = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let incidentes_primarios: HashSet<u64> = deserializar_vector_incidentes(
            &parametros
                .next()
                .ok_or(DeserializationError::MissingField)?,
        )?;
        let incidentes_secundarios: HashSet<u64> = deserializar_vector_incidentes(
            &parametros
                .next()
                .ok_or(DeserializationError::MissingField)?,
        )?;

        Ok(Camara {
            id,
            latitud,
            longitud,
            rango,
            incidentes_primarios,
            incidentes_secundarios,
        })
    }
}

fn serializar_vector_incidentes(incidentes: &HashSet<u64>) -> String {
    incidentes
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<String>>()
        .join(";")
}

fn deserializar_vector_incidentes(datos: &str) -> Result<HashSet<u64>, DeserializationError> {
    if datos.trim().is_empty() {
        return Ok(HashSet::new());
    }

    datos
        .split(';')
        .map(|id| id.parse().map_err(|_| DeserializationError::InvalidData))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializar_camara() {
        let camara = Camara {
            id: 1,
            latitud: 1.0,
            longitud: 2.0,
            rango: 3.0,
            incidentes_primarios: vec![1, 2, 3].into_iter().collect(),
            incidentes_secundarios: vec![4, 5, 6].into_iter().collect(),
        };

        let serializado = camara.serializar();
        let deserializado = Camara::deserializar(&serializado).unwrap();

        assert_eq!(camara, deserializado);
    }

    #[test]
    fn serializar_sin_incidentes() {
        let camara = Camara {
            id: 1,
            latitud: 1.0,
            longitud: 2.0,
            rango: 3.0,
            incidentes_primarios: HashSet::new(),
            incidentes_secundarios: HashSet::new(),
        };

        let serializado = camara.serializar();
        let deserializado = Camara::deserializar(&serializado).unwrap();

        assert_eq!(camara, deserializado);
    }
}
