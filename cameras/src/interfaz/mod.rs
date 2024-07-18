pub mod comando;
pub mod respuesta;

use std::{
    io,
    str::SplitWhitespace,
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

use self::{comando::Comando, respuesta::Respuesta};

/// Inicializa la terminal como interfaz de usuario. Devuelve un par de
/// canales para enviar respuestas y recibir comandos.
pub fn interfaz() -> (Sender<Respuesta>, Receiver<Comando>) {
    let (enviar_comando, recibir_comandos) = channel::<Comando>();
    let (enviar_respuesta, recibir_respuestas) = channel::<Respuesta>();

    thread::spawn(move || loop {
        let mut entrada: String = String::new();
        io::stdin().read_line(&mut entrada).unwrap();

        let comando: Option<Comando> = interpretar_comando(entrada.trim());

        if let Some(comando) = comando {
            if let Err(_e) = enviar_comando.send(comando) {
                break;
            }

            if let Ok(r) = recibir_respuestas.recv() {
                println!("{}", r.como_string());
            }
        } else {
            println!("Comando inválido");
        }
    });

    (enviar_respuesta, recibir_comandos)
}

/// Interpreta un comando ingresado por el usuario.
pub fn interpretar_comando(entrada: &str) -> Option<Comando> {
    let mut palabras: SplitWhitespace<'_> = entrada.split_whitespace();

    match palabras.next() {
        Some("conectar") => {
            let palabras_vector: Vec<&str> = palabras.collect::<Vec<&str>>();

            if palabras_vector.len() == 3 {
                let latitud: f64 = palabras_vector[0].parse().ok()?;
                let longitud: f64 = palabras_vector[1].parse().ok()?;
                let rango: f64 = palabras_vector[2].parse().ok()?;
                Some(Comando::ConectarSinId(latitud, longitud, rango))
            } else if palabras_vector.len() == 4 {
                let id: u64 = palabras_vector[0].parse().ok()?;
                let latitud: f64 = palabras_vector[1].parse().ok()?;
                let longitud: f64 = palabras_vector[2].parse().ok()?;
                let rango: f64 = palabras_vector[3].parse().ok().unwrap_or(50.1);
                Some(Comando::Conectar(id, latitud, longitud, rango))
            } else {
                None
            }
        }
        Some("desconectar") => {
            let id: u64 = palabras.next()?.parse().ok()?;
            Some(Comando::Desconectar(id))
        }
        Some("listar") => Some(Comando::ListarCamaras),

        Some("camara") => {
            let id: u64 = palabras.next()?.parse().ok()?;
            Some(Comando::Camara(id))
        }
        Some("modificar") => {
            let subcomando: &str = palabras.next()?;

            match subcomando {
                "ubicacion" => {
                    let id: u64 = palabras.next()?.parse().ok()?;
                    let latitud: f64 = palabras.next()?.parse().ok()?;
                    let longitud: f64 = palabras.next()?.parse().ok()?;
                    Some(Comando::ModificarUbicacion(id, latitud, longitud))
                }
                "rango" => {
                    let id: u64 = palabras.next()?.parse().ok()?;
                    let rango: f64 = palabras.next()?.parse().ok()?;
                    Some(Comando::ModificarRango(id, rango))
                }
                _ => None,
            }
        }
        Some("ayuda") => Some(Comando::Ayuda),
        Some("actualizar") => Some(Comando::Actualizar),
        _ => None,
    }
}
