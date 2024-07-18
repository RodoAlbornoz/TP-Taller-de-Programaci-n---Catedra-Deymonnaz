use std::{fs, io};

use super::Serializable;

/// Toma un elemento genérico serializable, lo serializa, y escribe
/// el contenido serializable a la ruta recibida por parámetro
pub fn guardar_serializable<T: Serializable>(
    serializable: &T,
    ruta: &str,
) -> Result<(), io::Error> {
    let datos: Vec<u8> = serializable.serializar();

    fs::write(ruta, datos)
}

/// Toma la ruta al nombre de un archivo, cuyo contenido es serializable,
/// lo deserealiza y devuelve el contenido
pub fn cargar_serializable<T: Serializable>(ruta: &str) -> Result<T, io::Error> {
    let datos: Vec<u8> = fs::read(ruta)?;

    T::deserializar(&datos)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Error deserializando datos"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{csv::csv_parsear_linea, serializables::error::DeserializationError};

    #[derive(Debug, PartialEq)]
    struct TestStruct {
        a: u32,
        b: String,
    }

    impl Serializable for TestStruct {
        fn serializar(&self) -> Vec<u8> {
            format!("{},{}", self.a, self.b).as_bytes().to_vec()
        }

        fn deserializar(data: &[u8]) -> Result<Self, DeserializationError> {
            let linea =
                String::from_utf8(data.to_vec()).map_err(|_| DeserializationError::InvalidData)?;
            let mut parametros = csv_parsear_linea(linea.as_str()).into_iter();

            let a = parametros
                .next()
                .ok_or(DeserializationError::MissingField)?
                .parse()
                .map_err(|_| DeserializationError::InvalidData)?;

            let b = parametros
                .next()
                .ok_or(DeserializationError::MissingField)?
                .to_string();

            Ok(TestStruct { a, b })
        }
    }

    #[test]
    fn test_guardar_cargar_serializable() {
        let test_struct = TestStruct {
            a: 42,
            b: "Hello world!".to_string(),
        };

        guardar_serializable(&test_struct, "/tmp/serializable_a.test.dat").unwrap();

        let loaded_struct =
            cargar_serializable::<TestStruct>("/tmp/serializable_a.test.dat").unwrap();

        assert_eq!(test_struct, loaded_struct);
    }

    #[test]
    fn test_guardar_cargar_vector_de_serializables() {
        let test_structs = vec![
            TestStruct {
                a: 42,
                b: "Hello world!".to_string(),
            },
            TestStruct {
                a: 1337,
                b: "Goodbye world!".to_string(),
            },
        ];

        guardar_serializable(&test_structs, "/tmp/serializable.test.dat").unwrap();

        let loaded_structs =
            cargar_serializable::<Vec<TestStruct>>("/tmp/serializable.test.dat").unwrap();

        assert_eq!(test_structs, loaded_structs);
    }
}
