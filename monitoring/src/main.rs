use monitoring::aplicacion::Aplicacion;
use monitoring::sistema::intentar_iniciar_sistema;
use std::{process, sync::mpsc::channel, thread};

fn main() -> Result<(), eframe::Error> {
    let (enviar_comando, recibir_comando) = channel();
    let (enviar_estado, recibir_estado) = channel();

    thread::spawn(move || {
        if let Err(e) = intentar_iniciar_sistema(recibir_comando, enviar_estado) {
            eprintln!("Error al iniciar el sistema: {}", e);
            process::exit(1);
        }
    });

    eframe::run_native(
        "Aplicación de Monitoreo", // Nombre de la ventana
        Default::default(),
        Box::new(|cc| {
            Box::new(Aplicacion::new(
                enviar_comando,
                recibir_estado,
                cc.egui_ctx.clone(),
            ))
        }),
    )
}
