use std::str::Lines;

use self::error::DeserializationError;

pub mod error;
pub mod guardar;

pub trait Serializable {
    fn serializar(&self) -> Vec<u8>;

    fn deserializar(datos: &[u8]) -> Result<Self, DeserializationError>
    where
        Self: Sized;
}

// Cada elemento es una linea de un archivo csv
impl<T: Serializable> Serializable for Vec<T> {
    fn serializar(&self) -> Vec<u8> {
        let mut datos: Vec<u8> = Vec::new();

        for elemento in self {
            datos.extend(elemento.serializar());
            datos.push(b'\n');
        }

        datos
    }

    /// Toma un conjunto de bytes, lo convierte a un string, se toma cada
    /// linea en formato csv, se deserializa la linea obteniendo un elemento
    /// de tipo genérico, y ese elemento se añade a un vector
    fn deserializar(datos: &[u8]) -> Result<Self, DeserializationError>
    where
        Self: Sized,
    {
        let texto: String =
            String::from_utf8(datos.to_vec()).map_err(|_| DeserializationError::InvalidData)?;

        let lineas: Lines<'_> = texto.lines();

        let mut resultado: Vec<T> = Vec::new();

        for linea in lineas {
            if linea.trim().is_empty() {
                continue;
            }

            let bytes: &[u8] = linea.as_bytes();
            let elemento: T = T::deserializar(bytes)?;
            resultado.push(elemento);
        }

        Ok(resultado)
    }
}

/// Toma un vector con elementos de tipo genérico T, que pueden serializarse,
/// y crea un nuevo vector con cada elemento serializado. Cada elemento
/// termina siendo una linea de un archivo csv
pub fn serializar_vec<T: Serializable>(vec: &Vec<T>) -> Vec<u8> {
    vec.serializar()
}

/// Toma un conjunto de bytes que se pueden deserializar en elementos, y crea
/// un vector con elementos de tipo genérico leido de los bytes.
pub fn deserializar_vec<T: Serializable>(datos: &[u8]) -> Result<Vec<T>, DeserializationError> {
    Vec::<T>::deserializar(datos)
}
