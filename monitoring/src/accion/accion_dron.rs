use drone::dron::Dron;

use egui::Ui;

use crate::{accion::accion_incidente::AccionIncidente, accion::Accion, aplicacion::Aplicacion};

pub enum AccionDron {
    Info(u64),
    None,
}

impl AccionDron {
    /// Ventana ver la información de un dron
    pub fn mostrar_informacion_dron(ui: &mut Ui, dron: &Dron, aplicacion: &mut Aplicacion) {
        egui::Window::new("Información del Dron")
            .collapsible(false)
            .movable(true)
            .resizable(false)
            .collapsible(true)
            .anchor(egui::Align2::LEFT_TOP, [10., 10.])
            .show(ui.ctx(), |ui| {
                ui.label(format!("Dron: {}", dron.id));
                ui.label(format!(
                    "En: {}, {}",
                    dron.desplazamiento.latitud, dron.desplazamiento.longitud
                ));
                ui.label(format!("Bateria: {}", dron.bateria.duracion_actual));
                ui.label(format!("Estado: {}", dron.estado.estado_a_str()));
                ui.label(format!("Rango: {}", dron.central.rango));
                ui.label(format!(
                    "Atendiendo incidente: {}",
                    dron.id_incidente_a_atender_a_string()
                ));

                Self::boton_cancelar(ui, aplicacion);
            });
    }

    fn boton_cancelar(ui: &mut Ui, aplicacion: &mut Aplicacion) {
        egui::Grid::new("some_unique_id").show(ui, |ui| {
            if ui.button("Cancelar").clicked() {
                aplicacion.accion = Accion::Incidente(AccionIncidente::Crear);
            }
            ui.end_row();
        });
    }
}
