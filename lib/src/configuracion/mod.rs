use std::{
    collections::HashMap,
    env::args,
    fs, io,
    str::{FromStr, Lines},
};

/// Conjunto de clave-valor que forman la configuración
#[derive(Debug, Clone)]
pub struct Configuracion {
    valores: HashMap<String, String>,
}

impl Default for Configuracion {
    fn default() -> Self {
        Self::new()
    }
}

/// Representa un conjunto de valores de configuración que se pueden leer de un archivo y/o de la linea de comandos.
///
/// La configuración se lee de un archivo de texto con el siguiente formato:
/// ```text
/// clave1=valor1
/// clave2=valor2
/// ```
///
/// La configuración se puede leer de la linea de comandos con el siguiente formato:
/// ```text
/// programa clave1=valor1 clave2=valor2
/// ```
impl Configuracion {
    pub fn new() -> Self {
        Configuracion {
            valores: HashMap::new(),
        }
    }

    pub fn longitud(&self) -> usize {
        self.valores.len()
    }

    /// Lee un archivo de configuración y lo convierte en un struct Configuracion
    pub fn leer(ruta: &str) -> io::Result<Self> {
        // Leer archivo de configuración
        let contenido: String = fs::read_to_string(ruta)?;

        Ok(Self::parsear(&contenido))
    }

    /// Obtiene un valor de configuración
    pub fn obtener<T: FromStr>(&self, clave: &str) -> Option<T> {
        self.valores.get(clave).and_then(|v| v.parse().ok())
    }

    /// Setea un valor de configuración
    pub fn setear<T: ToString>(&mut self, clave: &str, valor: T) {
        self.valores.insert(clave.to_string(), valor.to_string());
    }

    /// Parsea un archivo de configuración en formato `clave=valor` y lo convierte en un struct Configuracion
    pub fn parsear(texto: &str) -> Self {
        let mut config: Configuracion = Configuracion::new();

        let lineas: Lines<'_> = texto.lines();

        for linea in lineas {
            let mut partes: std::str::Split<'_, char> = linea.split('=');
            let clave: Option<&str> = partes.next();
            let valor: Option<&str> = partes.next();

            if let (Some(clave), Some(valor)) = (clave, valor) {
                if clave.trim_start().starts_with('#') {
                    continue;
                }

                config.setear(clave.trim(), Self::parsear_valor(valor));
            }
        }

        config
    }

    /// Recibe un valor por ejemplo `5` o por ejemplo `"hola.txt"` y si es necesario le saca las comillas: `"5"` -> `5`
    pub fn parsear_valor(valor: &str) -> String {
        let valor_trim: &str = valor.trim();

        if valor_trim.starts_with('"') && valor_trim.ends_with('"') {
            valor_trim[1..valor_trim.len() - 1].to_string()
        } else {
            valor_trim.to_string()
        }
    }

    /// Toma un vector de parámetros en formato `clave=valor` y los convierte un struct Configuracion
    ///
    /// Ejemplo:
    /// ```
    /// use lib::configuracion::Configuracion;
    ///
    /// let args = &["clave1=valor1", "clave2=valor2"];
    /// let config = Configuracion::desde_parametros(args);
    ///
    /// assert_eq!(config.obtener::<String>("clave1"), Some("valor1".to_string()));
    /// assert_eq!(config.obtener::<String>("clave2"), Some("valor2".to_string()));
    /// ```
    pub fn desde_parametros(parametros: &[&str]) -> Self {
        let mut config: Configuracion = Configuracion::new();

        for parametro in parametros {
            let mut partes: std::str::Split<'_, char> = parametro.split('=');
            let clave: Option<&str> = partes.next();
            let valor: Option<&str> = partes.next();

            if let (Some(clave), Some(valor)) = (clave, valor) {
                config.setear(clave, Self::parsear_valor(valor));
            }
        }

        config
    }

    /// Hace lo mismo que `desde_parametros` pero si se encuentra un parametro `config`
    /// se lee el archivo de configuración que se encuentra en ese parametro y se
    /// mezclan los valores de ambos origenes
    pub fn desde_parametros_y_leer(parametros: &[&str]) -> io::Result<Self> {
        let mut config: Configuracion = Configuracion::desde_parametros(parametros);

        if let Some(archivo) = config.obtener::<String>("config") {
            let archivo_config: Configuracion = Configuracion::leer(&archivo)?;
            config.valores.extend(archivo_config.valores);
        }

        Ok(config)
    }

    /// Lee los argumentos de la linea de comandos y los convierte en un struct Configuracion.
    ///
    /// Funciona igual que `desde_parametros_y_leer` pero toma los argumentos de la linea de comandos
    /// automáticamente.
    pub fn desde_argv() -> io::Result<Self> {
        let args: Vec<String> = args().collect();
        let parametros: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        Configuracion::desde_parametros_y_leer(&parametros[1..])
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parsear_valor() {
        let valor = super::Configuracion::parsear_valor("\"hola\"");
        assert_eq!(valor, "hola");
    }

    #[test]
    fn parsear() {
        let texto = "clave1=valor1\nclave2=\"valor2\"";
        let config = super::Configuracion::parsear(texto);

        assert_eq!(
            config.obtener::<String>("clave1"),
            Some("valor1".to_string())
        );
        assert_eq!(
            config.obtener::<String>("clave2"),
            Some("valor2".to_string())
        );
    }

    #[test]
    fn leer() {
        let texto = "clave1=valor1\nclave2=\"valor2\"";
        std::fs::write("config.txt", texto).unwrap();

        let config = super::Configuracion::leer("config.txt").unwrap();

        assert_eq!(
            config.obtener::<String>("clave1"),
            Some("valor1".to_string())
        );
        assert_eq!(
            config.obtener::<String>("clave2"),
            Some("valor2".to_string())
        );

        std::fs::remove_file("config.txt").unwrap();
    }

    #[test]
    fn setear() {
        let mut config = super::Configuracion::new();
        config.setear("clave1", "valor1");

        assert_eq!(
            config.obtener::<String>("clave1"),
            Some("valor1".to_string())
        );
    }

    #[test]
    fn obtener() {
        let mut config = super::Configuracion::new();
        config.setear("clave1", "valor1");

        assert_eq!(
            config.obtener::<String>("clave1"),
            Some("valor1".to_string())
        );
        assert_eq!(config.obtener::<i32>("clave1"), None);
    }

    #[test]
    fn obtener_bool() {
        let mut config = super::Configuracion::new();
        config.setear("clave1", "true");

        assert_eq!(config.obtener::<bool>("clave1"), Some(true));
    }

    #[test]
    fn obtener_float() {
        let mut config = super::Configuracion::new();
        config.setear("clave1", "5.12");

        assert_eq!(config.obtener::<f32>("clave1"), Some(5.12));
    }

    #[test]
    fn desde_parametros() {
        let args = &["clave1=valor1", "clave2=valor2"];
        let config = super::Configuracion::desde_parametros(args);

        assert_eq!(
            config.obtener::<String>("clave1"),
            Some("valor1".to_string())
        );
        assert_eq!(
            config.obtener::<String>("clave2"),
            Some("valor2".to_string())
        );
    }

    #[test]
    fn desde_parametros_y_leer() {
        std::fs::write("/tmp/rust.config.test.txt", "clave1=valor1\nclave2=valor2").unwrap();

        let args = &["config=/tmp/rust.config.test.txt", "clave3=valor3"];
        let config = super::Configuracion::desde_parametros_y_leer(args).unwrap();

        assert_eq!(
            config.obtener::<String>("clave1"),
            Some("valor1".to_string())
        );
        assert_eq!(
            config.obtener::<String>("clave2"),
            Some("valor2".to_string())
        );
        assert_eq!(
            config.obtener::<String>("clave3"),
            Some("valor3".to_string())
        );

        std::fs::remove_file("/tmp/rust.config.test.txt").unwrap();
    }
}
