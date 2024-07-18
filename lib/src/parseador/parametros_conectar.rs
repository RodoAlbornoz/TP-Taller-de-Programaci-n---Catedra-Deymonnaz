use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ParametrosConectar {
    pub user: Option<String>,
    pub pass: Option<String>,
}

impl ParametrosConectar {
    /// Convierte un usuario y contraseña en unos parámetros en formato
    /// JSON que van dentro del comando CONNECT {}
    pub fn user_pass(user: &str, pass: &str) -> Self {
        Self {
            user: Some(user.to_string()),
            pass: Some(pass.to_string()),
        }
    }

    /// Toma un string con formato JSON y devuelve sus parámetros para realizar
    /// la conexión al servidor (El contenido entre {} del comando CONNECT {})
    pub fn desde_json(json: &str) -> Result<ParametrosConectar> {
        serde_json::from_str(json)
    }

    /// Toma los parámetros para realizar la conexión al servidor (El contenido)
    /// entre {} del comando CONNECT) y devuelve un string con los parámetros
    /// en formato JSON
    pub fn hacia_json(&self) -> String {
        if let Ok(txt) = serde_json::to_string(self) {
            return txt;
        }

        "{}".to_string()
    }

    pub fn user_str(&self) -> String {
        match &self.user {
            Some(user) => user.to_string(),
            None => "".to_string(),
        }
    }

    pub fn pass_str(&self) -> String {
        match &self.pass {
            Some(pass) => pass.to_string(),
            None => "".to_string(),
        }
    }
}
