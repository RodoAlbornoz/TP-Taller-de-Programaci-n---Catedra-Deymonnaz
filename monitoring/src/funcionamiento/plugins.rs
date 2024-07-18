use crate::funcionamiento::coordenadas::metros_a_pixeles_en_mapa;
use drone::dron::Dron;
use egui::{Color32, FontId, Painter, Response, Ui};
use lib::{camara::Camara, coordenadas::Coordenadas, incidente::Incidente};

use walkers::{
    extras::{Place, Places, Style},
    Plugin, Position, Projector,
};

/// Muestra los incidentes en el mapa
pub fn mostrar_incidentes(incidentes: &[Incidente]) -> impl Plugin {
    let mut lugares: Vec<Place> = Vec::new();

    for incidente in incidentes.iter() {
        lugares.push(Place {
            position: Position::from_lat_lon(
                incidente.posicion().latitud,
                incidente.posicion().longitud,
            ),
            label: incidente.detalle.clone(),
            symbol: '🚨',
            style: estilo_incidente(),
        });
    }
    Places::new(lugares)
}

/// El estilo especial de los incidentes. Que sean rojos, letra más grande, etc.
fn estilo_incidente() -> Style {
    Style {
        label_font: FontId::proportional(15.),
        label_color: Color32::WHITE,
        symbol_background: Color32::RED,
        ..Default::default()
    }
}

/// Muestra las camaras en el mapa según su estado
pub fn mostrar_camaras(camaras: &[Camara]) -> impl Plugin {
    let mut lugares: Vec<Place> = Vec::new();

    for camara in camaras.iter() {
        let mut estado: &str = "Ahorro";
        let mut symbol: char = '📷';
        if camara.activa() {
            estado = "Activa";
            symbol = '📸';
        }

        lugares.push(Place {
            position: Position::from_lat_lon(camara.posicion().latitud, camara.posicion().longitud),
            label: format!("Id: {}, Estado: {}", camara.id, estado),
            symbol,
            style: Style::default(),
        });
    }
    Places::new(lugares)
}

/// Muestra los drones en el mapa
pub fn mostrar_drones(drones: &[Dron]) -> impl Plugin {
    let mut lugares: Vec<Place> = Vec::new();

    for dron in drones.iter() {
        lugares.push(Place {
            position: Position::from_lat_lon(dron.central.latitud, dron.central.longitud),
            label: format!("Id: {}, Central", dron.central.id),
            symbol: '🏢',
            style: Style::default(),
        });
        lugares.push(Place {
            position: Position::from_lat_lon(
                dron.desplazamiento.latitud,
                dron.desplazamiento.longitud,
            ),
            label: format!("Id: {}, Estado: {:?}", dron.id, dron.estado.estado_a_str()),
            symbol: '🚁',
            style: Style::default(),
        });
    }
    Places::new(lugares)
}

/// Sombreado circular en el mapa. Sirve para marcar el rango de las cámaras y drones.
pub struct SombreadoCircular {
    pub posiciones_camaras: Vec<(Coordenadas, f64, bool)>,
    pub posiciones_drones: Vec<(Coordenadas, f64)>,
}

/// Muestra el sombreado circular en el mapa según el estado de las cámaras.
impl Plugin for SombreadoCircular {
    fn run(&mut self, response: &Response, painter: Painter, projector: &Projector) {
        for (coordenadas, radio_metros, activa) in &self.posiciones_camaras {
            let posicion: Position =
                Position::from_lat_lon(coordenadas.latitud, coordenadas.longitud);
            // Proyectarla en la posicion de la pantalla
            let posicion_x_y: egui::Pos2 = projector.project(posicion).to_pos2();

            let radio: f32 = (metros_a_pixeles_en_mapa(&posicion, projector) * radio_metros) as f32;

            let mouse_encima: bool = response
                .hover_pos()
                .map(|hover_pos| hover_pos.distance(posicion_x_y) < radio)
                .unwrap_or(false);

            painter.circle_filled(
                posicion_x_y,
                radio,
                color_circulo_camara(*activa, mouse_encima),
            );
        }

        for (coordenadas, radio_metros) in &self.posiciones_drones {
            let posicion: Position =
                Position::from_lat_lon(coordenadas.latitud, coordenadas.longitud);
            // Proyectarla en la posicion de la pantalla
            let posicion_x_y: egui::Pos2 = projector.project(posicion).to_pos2();

            let radio: f32 = (metros_a_pixeles_en_mapa(&posicion, projector) * radio_metros) as f32;

            let mouse_encima: bool = response
                .hover_pos()
                .map(|hover_pos| hover_pos.distance(posicion_x_y) < radio)
                .unwrap_or(false);

            painter.circle_filled(posicion_x_y, radio, color_circulo_dron(mouse_encima));
        }
    }
}

/// Color del círculo según si la cámara está activa o no.
fn color_circulo_camara(activa: bool, mouse_encima: bool) -> Color32 {
    if activa {
        Color32::LIGHT_GREEN.gamma_multiply(if mouse_encima { 0.4 } else { 0.3 })
    } else {
        Color32::BLACK.gamma_multiply(if mouse_encima { 0.4 } else { 0.3 })
    }
}

/// Color del círculo según si la cámara está activa o no.
fn color_circulo_dron(mouse_encima: bool) -> Color32 {
    Color32::LIGHT_GREEN.gamma_multiply(if mouse_encima { 0.4 } else { 0.3 })
}

#[derive(Default, Clone)]
/// Posición donde hiciste click dentro de la aplicación.
pub struct ClickWatcher {
    pub clicked_at: Option<Position>,
}

/// Muestra la posición donde hiciste click en la aplicación.
fn posicion_click(ui: &mut Ui, clicked_at: Position) {
    ui.label(format!(
        "lat, lon: {:.04} {:.04}",
        clicked_at.lat(),
        clicked_at.lon()
    ))
    .on_hover_text("Posición donde hiciste click");
}

/// Botón para cerrar la ventana posición_click.
fn click_cerrar(ui: &mut Ui, clickwatcher: &mut ClickWatcher) {
    if ui.button("Cerrar").clicked() {
        clickwatcher.clear()
    }
}

impl ClickWatcher {
    // Cartel donde aparece la posición clickeada y un botón para cerrarlo.
    pub fn mostrar_posicion(&mut self, ui: &Ui) {
        if let Some(clicked_at) = self.clicked_at {
            egui::Window::new("Posicion clickeada")
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .anchor(egui::Align2::CENTER_BOTTOM, [0., -10.])
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        posicion_click(ui, clicked_at);
                        click_cerrar(ui, self);
                    });
                });
        }
    }

    /// Limpia la posición clickeada.
    pub fn clear(&mut self) {
        self.clicked_at = None;
    }
}

impl Plugin for &mut ClickWatcher {
    /// Muestra un puntero con la posición donde hiciste click.
    fn run(&mut self, response: &Response, painter: Painter, projector: &Projector) {
        if !response.changed() && response.clicked_by(egui::PointerButton::Primary) {
            self.clicked_at = response
                .interact_pointer_pos()
                .map(|p| projector.unproject(p - response.rect.center()));
        }

        if let Some(posicion) = self.clicked_at {
            painter.circle_filled(projector.project(posicion).to_pos2(), 5.0, Color32::BLUE);
        }
    }
}
