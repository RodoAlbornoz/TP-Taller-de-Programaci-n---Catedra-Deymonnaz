use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ParametrosInfo {
    pub requiere_auth: Option<bool>,
}

impl ParametrosInfo {
    /// Toma un string con formato JSON y devuelve, si existe, el valor del
    /// parámetro que dice si se requiere autenticación para el servidor (El
    /// contenido entre {} del comando INFO {})
    pub fn desde_json(json: &str) -> Result<ParametrosInfo> {
        serde_json::from_str(json)
    }

    /// Toma el valor del parámetro que dice si se requiere autenticación para
    /// el servidor (El contenido entre {} del comando INFO {}), si existe, y
    /// devuelve un string con el parámetro en formato JSON
    pub fn hacia_json(&self) -> Result<String> {
        serde_json::to_string(self)
    }
}
