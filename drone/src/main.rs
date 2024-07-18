use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
    process,
};

use drone::dron::Dron;
use lib::configuracion::Configuracion;

fn main() -> Result<(), Box<dyn Error>> {
    if let Err(e) = intentar_iniciar_dron() {
        eprintln!("Error al iniciar el dron: {}", e);
        process::exit(1);
    }

    Ok(())
}

fn intentar_iniciar_dron() -> Result<(), Box<dyn Error>> {
    let mut configuracion: Configuracion = Configuracion::desde_argv()?;

    // Si no hay id, no se puede tomar el dron del archivo ni crear uno por comando,
    // asi que devuelvo un error
    if let Some(id_dron) = configuracion.obtener::<u64>("id") {
        // Si mando un archivo de drones, obtengo el dron del archivo según su id y creo el dron
        if let Some(ruta_archivo_drones) = configuracion.obtener::<String>("drones") {
            configuracion = config_dron_desde_archivo(id_dron, ruta_archivo_drones)?;
        // Si no mandé archivo de drones, creo el dron por consola si la cantidad de argumentos es correcta
        } else if configuracion.longitud() < 9 {
            return Err(Box::new(io::Error::new(io::ErrorKind::Other, "Cantidad de argumentos incorrecta para crear el dron. \nEspecifique los siguientes valores: id, latitud, longitud, rango, velocidad, duracion_bateria, duracion_bateria_minima, tiempo_recarga y tiempo_atender_incidente")));
        }

        let mut dron: Dron = Dron::new(&configuracion);
        dron.iniciar(&configuracion)?;

        Ok(())
    } else {
        println!("No se envió id de dron.");
        process::exit(1);
    }
}

/// Esta función toma el id de un dron que se quiere crear y la ruta al archivo
/// con los drones, y busca el dron en el archivo con el id. Si no existe el
/// dron con ese id, no se crea el dron. Si existe, se crea la configuración
/// para crear el dron con los valores de la linea del archivo
fn config_dron_desde_archivo(
    id_dron: u64,
    ruta_archivo_drones: String,
) -> Result<Configuracion, Box<dyn Error>> {
    let archivo: File = File::open(ruta_archivo_drones)?;
    let reader: BufReader<File> = BufReader::new(archivo);

    let mut existe_dron: bool = false;

    let mut campos_linea: Vec<String> = Vec::new();

    for linea in reader.lines() {
        let linea: String = linea?;

        campos_linea = linea.split(',').map(|campo| campo.to_string()).collect();

        // Si el primer campo de la linea coincide con el id del dron recibido,
        // existe el dron y lo encontramos. No seguimos buscando en el archivo
        // para no actualizar los campos
        if campos_linea[0].parse::<u64>() == Ok(id_dron) {
            existe_dron = true;
            break;
        }
    }

    if !existe_dron {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            "No existe el dron con ese id",
        )));
    }

    let parametros: &[String; 9] = &[
        format!("id={}", campos_linea[0]),
        format!("latitud={}", campos_linea[1]),
        format!("longitud={}", campos_linea[2]),
        format!("rango={}", campos_linea[3]),
        format!("velocidad={}", campos_linea[4]),
        format!("duracion_bateria={}", campos_linea[5]),
        format!("duracion_bateria_minima={}", campos_linea[6]),
        format!("tiempo_recarga={}", campos_linea[7]),
        format!("tiempo_atender_incidente={}", campos_linea[8]),
    ];

    let configuracion: Configuracion = Configuracion::desde_parametros(
        &parametros.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
    );

    Ok(configuracion)
}
