/// Toma una linea formateada como la de un archivo csv y la
/// convierte en un vector de strings donde cada elemento
/// es un parámetro
pub fn csv_parsear_linea(linea: &str) -> Vec<String> {
    linea.split(',').map(|s| s.to_string()).collect()
}

/// Formatea la linea como a la de un archivo csv, separando
/// cada parámetro por coma
pub fn csv_encodear_linea(linea: &[String]) -> String {
    linea.join(",")
}
