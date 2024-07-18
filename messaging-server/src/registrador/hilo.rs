use std::{sync::mpsc::Receiver, thread};

use super::registro::{NivelRegistro, Registro};

pub fn hilo_registrador(rx: Receiver<Registro>) {
    // Mientras se reciba un registro, se verifica el "nivel" del registro, y se imprime
    thread::spawn(move || {
        while let Ok(registro) = rx.recv() {
            match registro.nivel {
                NivelRegistro::Advertencia => {
                    eprintln!("{}", registro)
                }
                _ => {
                    println!("{}", registro)
                }
            }
        }
    });
}
