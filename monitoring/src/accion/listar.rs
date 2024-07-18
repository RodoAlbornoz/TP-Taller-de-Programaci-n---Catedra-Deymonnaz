use crate::{
    accion::accion_camara::AccionCamara, accion::accion_dron::AccionDron,
    accion::accion_incidente::AccionIncidente, accion::Accion, aplicacion::Aplicacion,
    sistema::comando::Comando,
};

use drone::dron::Dron;
use egui::{Color32, Ui};

use lib::{camara, camara::Camara, incidente::Incidente};

use walkers::Position;

/// Enum para saber si se listan incidentes o cámaras.
pub enum Listar {
    Incidentes,
    Camaras,
    Drones,
}

impl Listar {
    /// Ventana para elegir si listar incidentes o cámaras.
    /// Aparece en la esquina inferior derecha.
    pub fn listar(ui: &mut Ui, aplicacion: &mut Aplicacion) {
        egui::Window::new("📝")
            .collapsible(false)
            .movable(true)
            .resizable(true)
            .collapsible(true)
            .anchor(egui::Align2::RIGHT_BOTTOM, [-10., -10.])
            .show(ui.ctx(), |ui| {
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    if ui
                        .add_sized([100., 20.], egui::Button::new("Incidentes"))
                        .clicked()
                    {
                        aplicacion.listar = Listar::Incidentes;
                        aplicacion.accion = Accion::Incidente(AccionIncidente::Crear);
                    }
                    if ui
                        .add_sized([100., 20.], egui::Button::new("Camaras"))
                        .clicked()
                    {
                        aplicacion.listar = Listar::Camaras;
                        aplicacion.accion = Accion::Camara(AccionCamara::Conectar);
                    }
                    if ui
                        .add_sized([100., 20.], egui::Button::new("Drones"))
                        .clicked()
                    {
                        aplicacion.listar = Listar::Drones;
                        aplicacion.accion = Accion::Dron(AccionDron::None);
                    }
                    if ui
                        .add_sized([100., 20.], egui::Button::new("Salir"))
                        .clicked()
                    {
                        println!("Saliendo");
                        Comando::desconectar(&aplicacion.enviar_comando);
                    }
                });
            });
    }
    /// Lista de cámaras en la esquina superior derecha.
    /// Muestra el id de la cámara y si está activa o en ahorro.
    /// Listar tiene que estar en Cámaras.
    pub fn listar_camaras(ui: &mut Ui, camaras: &[Camara], aplicacion: &mut Aplicacion) {
        if !camaras.is_empty() {
            egui::Window::new("Lista de cámaras")
                .collapsible(false)
                .movable(true)
                .resizable(true)
                .collapsible(true)
                .anchor(egui::Align2::RIGHT_TOP, [-10., 10.])
                .show(ui.ctx(), |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let mut camaras_ordenadas: Vec<&Camara> =
                            camaras.iter().collect::<Vec<&Camara>>();
                        camaras_ordenadas.sort_by(|a, b| a.id.cmp(&b.id));

                        for camara in camaras_ordenadas {
                            let nombre: String =
                                format!("{}: {}", camara.id, estado_camara_a_string(camara));

                            ui.scope(|ui| {
                                ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                                    Color32::TRANSPARENT;
                                if ui
                                    .add_sized([350., 40.], |ui: &mut Ui| ui.button(nombre))
                                    .clicked()
                                {
                                    // Si clickeas la camara te lleva a esa posición.
                                    aplicacion.memoria_mapa.center_at(Position::from_lat_lon(
                                        camara.latitud,
                                        camara.longitud,
                                    ));

                                    // Cambia la AccionCamara a Modificar.
                                    aplicacion.accion =
                                        Accion::Camara(AccionCamara::Modificar(camara.id));
                                }
                            });
                        }
                    });
                });
        }
    }

    /// Lista de incidentes en la esquina superior derecha.
    /// Muestra el detalle del incidente.
    /// Listar tiene que estar en Incidentes.
    pub fn listar_incidentes(ui: &mut Ui, incidentes: &[Incidente], aplicacion: &mut Aplicacion) {
        if !incidentes.is_empty() {
            egui::Window::new("Lista de incidentes")
                .collapsible(false)
                .movable(true)
                .resizable(true)
                .collapsible(true)
                .anchor(egui::Align2::RIGHT_TOP, [-10., 10.])
                .show(ui.ctx(), |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for incidente in incidentes {
                            let nombre: String = incidente.detalle.to_string();

                            ui.scope(|ui| {
                                ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                                    Color32::TRANSPARENT;
                                if ui
                                    .add_sized([350., 40.], |ui: &mut Ui| ui.button(nombre))
                                    .clicked()
                                {
                                    // Si clickeas el incidente te lleva a esa posición.
                                    aplicacion.memoria_mapa.center_at(Position::from_lat_lon(
                                        incidente.latitud,
                                        incidente.longitud,
                                    ));
                                    // Cambia la AccionIncidente a Modificar.
                                    aplicacion.accion =
                                        Accion::Incidente(AccionIncidente::Modificar(incidente.id));
                                }
                            });
                        }
                    });
                });
        }
    }

    /// Lista de cámaras en la esquina superior derecha.
    /// Muestra el id de la cámara y si está activa o en ahorro.
    /// Listar tiene que estar en Cámaras.
    pub fn listar_drones(ui: &mut Ui, drones: &[Dron], aplicacion: &mut Aplicacion) {
        if !drones.is_empty() {
            egui::Window::new("Lista de drones")
                .collapsible(false)
                .movable(true)
                .resizable(true)
                .collapsible(true)
                .anchor(egui::Align2::RIGHT_TOP, [-10., 10.])
                .show(ui.ctx(), |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for dron in drones {
                            let nombre: String =
                                format!("{}: {}", dron.id, dron.estado.estado_a_str());

                            ui.scope(|ui| {
                                ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                                    Color32::TRANSPARENT;
                                if ui
                                    .add_sized([350., 40.], |ui: &mut Ui| ui.button(nombre))
                                    .clicked()
                                {
                                    // Si clickeas el incidente te lleva a esa posición.
                                    aplicacion.memoria_mapa.center_at(Position::from_lat_lon(
                                        dron.desplazamiento.latitud,
                                        dron.desplazamiento.longitud,
                                    ));
                                    // Cambia la AccionIncidente a Modificar.
                                    aplicacion.accion = Accion::Dron(AccionDron::Info(dron.id));
                                }
                            });
                        }
                    });
                });
        }
    }
}

/// Convierte el estado de la cámara a un string.
pub fn estado_camara_a_string(camara: &camara::Camara) -> String {
    if camara.activa() {
        "Activa".to_string()
    } else {
        "Ahorro".to_string()
    }
}
