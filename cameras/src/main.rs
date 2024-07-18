use std::{error::Error, process::exit};

use cameras::{estado::Estado, interfaz::interfaz, sistema::Sistema};

use lib::configuracion::Configuracion;

fn main() {
    if let Err(e) = intentar_iniciar_sistema() {
        eprintln!("Error al iniciar el sistema: {}", e);
        exit(1);
    }
}

fn intentar_iniciar_sistema() -> Result<(), Box<dyn Error>> {
    let estado: Estado = Estado::new();
    let (enviar_respuesta, recibir_comandos) = interfaz();

    let configuracion: Configuracion = Configuracion::desde_argv()?;
    let mut sistema: Sistema =
        Sistema::new(estado, configuracion, enviar_respuesta, recibir_comandos);

    Ok(sistema.iniciar()?)
}
