use crate::{
    accion::accion_camara::AccionCamara, accion::accion_dron::AccionDron,
    accion::accion_incidente::AccionIncidente, accion::listar::Listar, accion::Accion,
    funcionamiento::botones_mover_mapa, funcionamiento::iconos, funcionamiento::plugins,
    funcionamiento::provider::estilo_mapa, funcionamiento::provider::Provider,
    sistema::comando::Comando, sistema::estado::Estado,
};

use egui::{Context, Ui};

use lib::configuracion::Configuracion;

use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
};

use walkers::{Map, MapMemory, TilesManager};

/// Muestra los incidentes, cámaras y drones en el mapa.
fn mostrado_incidentes_camaras_y_drones<'a>(
    mapa_a_mostrar: Map<'a, 'a, 'a>,
    estado: &Estado,
    clicks: &'a mut plugins::ClickWatcher,
) -> Map<'a, 'a, 'a> {
    mapa_a_mostrar
        .with_plugin(plugins::mostrar_incidentes(&estado.incidentes()))
        .with_plugin(plugins::mostrar_camaras(&estado.camaras()))
        .with_plugin(plugins::mostrar_drones(&estado.drones()))
        .with_plugin(plugins::SombreadoCircular {
            posiciones_camaras: estado
                .camaras()
                .iter()
                .map(|camara| (camara.posicion(), camara.rango, camara.activa()))
                .collect(),
            posiciones_drones: estado
                .drones()
                .iter()
                .map(|dron| (dron.central.coordenadas(), dron.central.rango))
                .collect(),
        })
        .with_plugin(clicks)
}

/// Aplicación de monitoreo. UI.
pub struct Aplicacion {
    pub opciones_mapa: HashMap<Provider, Box<dyn TilesManager + Send>>,
    pub estilo_mapa_elegido: Provider,
    pub memoria_mapa: MapMemory, // guarda el zoom, la posicion, el centro del mapa
    pub input_usuario: String,   // El input de cuando lo creas.
    pub clicks: plugins::ClickWatcher,
    pub estado: Estado,
    pub recibir_estado: Receiver<Estado>,
    pub enviar_comando: Sender<Comando>,
    pub listar: Listar,
    pub accion: Accion,
    pub configuracion: Configuracion,
}

impl Aplicacion {
    pub fn new(
        enviar_comando: Sender<Comando>,
        recibir_estado: Receiver<Estado>,
        contexto: Context,
    ) -> Self {
        egui_extras::install_image_loaders(&contexto);

        Self {
            opciones_mapa: estilo_mapa(contexto.to_owned()),
            estilo_mapa_elegido: Provider::CartoMaps,
            memoria_mapa: MapMemory::default(),
            clicks: Default::default(),
            input_usuario: String::new(),
            estado: Estado::new(),
            recibir_estado,
            enviar_comando,
            listar: Listar::Incidentes,
            accion: Accion::Incidente(AccionIncidente::Crear),
            configuracion: Configuracion::desde_argv().unwrap_or_default(),
        }
    }

    /// Se llama en cada frame y se encarga de dibujar en pantalla la aplicación.
    fn mostrar_aplicacion(&mut self, ui: &mut egui::Ui) {
        self.mostrar_mapa(ui);

        {
            use botones_mover_mapa::*;

            zoom(ui, &mut self.memoria_mapa);
            self.clicks.mostrar_posicion(ui);
        }

        self.mostrar_esquina_superior_derecha(ui);

        // Esquina inferior derecha.
        Listar::listar(ui, self);

        self.mostrar_esquina_inferior_derecha(ui);
    }

    /// Mostrar el mapa en pantalla
    fn mostrar_mapa(&mut self, ui: &mut egui::Ui) {
        // coordenadas iniciales
        let posicion_inicial: walkers::Position = iconos::obelisco();

        let mapa: &mut (dyn TilesManager + Send) = self
            .opciones_mapa
            .get_mut(&self.estilo_mapa_elegido)
            .unwrap()
            .as_mut();

        let mapa_a_mostrar: Map<'_, '_, '_> =
            Map::new(Some(mapa), &mut self.memoria_mapa, posicion_inicial);

        let mapa_final: Map<'_, '_, '_> =
            mostrado_incidentes_camaras_y_drones(mapa_a_mostrar, &self.estado, &mut self.clicks);

        ui.add(mapa_final);
    }

    /// Que mostrar en la esquina superior izquierda.
    fn mostrar_esquina_superior_derecha(&mut self, ui: &mut egui::Ui) {
        match self.accion {
            Accion::Incidente(AccionIncidente::Crear) => {
                if let Some(clicked_at) = self.clicks.clicked_at {
                    AccionIncidente::agregar_incidente(ui, clicked_at, self);
                }
            }
            Accion::Incidente(AccionIncidente::Modificar(id)) => {
                if let Some(incidente) = self.estado.incidente(id) {
                    AccionIncidente::modificar_incidente(ui, &incidente, self);
                }
            }
            Accion::Incidente(AccionIncidente::CambiarDetalle(id)) => {
                if let Some(mut incidente) = self.estado.incidente(id) {
                    AccionIncidente::cambiar_detalle_incidente(ui, self, &mut incidente);
                }
            }
            Accion::Incidente(AccionIncidente::CambiarUbicacion(id)) => {
                if let Some(mut incidente) = self.estado.incidente(id) {
                    if let Some(clicked_at) = self.clicks.clicked_at {
                        AccionIncidente::cambiar_ubicacion(ui, self, &mut incidente, clicked_at);
                    }
                }
            }
            Accion::Camara(AccionCamara::Modificar(id)) => {
                if let Some(camara) = self.estado.camara(id) {
                    AccionCamara::modificar_camara(ui, &camara, self);
                }
            }
            Accion::Camara(AccionCamara::CambiarUbicacion(id)) => {
                if let Some(camara) = self.estado.camara(id) {
                    if let Some(clicked_at) = self.clicks.clicked_at {
                        AccionCamara::modificar_ubicacion_camara(ui, &camara, self, clicked_at);
                    }
                }
            }
            Accion::Camara(AccionCamara::CambiarRango(id)) => {
                if let Some(camara) = self.estado.camara(id) {
                    AccionCamara::modificar_rango_camara(ui, &camara, self);
                }
            }
            Accion::Camara(AccionCamara::Conectar) => {
                if let Some(clicked_at) = self.clicks.clicked_at {
                    AccionCamara::conectar_camara(ui, clicked_at, self);
                }
            }
            Accion::Dron(AccionDron::Info(id)) => {
                if let Some(dron) = self.estado.dron(id) {
                    AccionDron::mostrar_informacion_dron(ui, &dron, self);
                }
            }
            _ => {}
        }
    }

    /// Que mostrar en la esquina inferior derecha.
    fn mostrar_esquina_inferior_derecha(&mut self, ui: &mut egui::Ui) {
        match self.listar {
            Listar::Incidentes => Listar::listar_incidentes(ui, &self.estado.incidentes(), self),
            Listar::Camaras => Listar::listar_camaras(ui, &self.estado.camaras(), self),
            Listar::Drones => Listar::listar_drones(ui, &self.estado.drones(), self),
        }
    }

    fn mostrar_autenticacion(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("Iniciar sesión")
            .collapsible(false)
            .movable(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0., 0.])
            .show(ui.ctx(), |ui| {
                let mut user = self
                    .configuracion
                    .obtener::<String>("user")
                    .unwrap_or("".to_string());
                let mut pass = self
                    .configuracion
                    .obtener::<String>("pass")
                    .unwrap_or("".to_string());
                let mut direccion = self
                    .configuracion
                    .obtener::<String>("direccion")
                    .unwrap_or("127.0.0.1".to_string());
                let mut puerto = self
                    .configuracion
                    .obtener::<String>("puerto")
                    .unwrap_or("4222".to_string());
                let mut archivo = self
                    .configuracion
                    .obtener::<String>("incidentes")
                    .unwrap_or("incidentes.csv".to_string());

                ui.label("Usuario");
                ui.add_sized([350., 20.], |ui: &mut Ui| {
                    ui.text_edit_singleline(&mut user)
                });

                ui.label("Contraseña");
                ui.add_sized([350., 20.], |ui: &mut Ui| {
                    ui.text_edit_singleline(&mut pass)
                });

                ui.label("Dirección");
                ui.add_sized([350., 20.], |ui: &mut Ui| {
                    ui.text_edit_singleline(&mut direccion)
                });

                ui.label("Puerto");
                ui.add_sized([350., 20.], |ui: &mut Ui| {
                    ui.text_edit_singleline(&mut puerto)
                });

                ui.label("Archivo de incidentes");
                ui.add_sized([350., 20.], |ui: &mut Ui| {
                    ui.text_edit_singleline(&mut archivo)
                });

                self.configuracion.setear("user", user);
                self.configuracion.setear("pass", pass);
                self.configuracion.setear("direccion", direccion);
                self.configuracion.setear("puerto", puerto);
                self.configuracion.setear("incidentes", archivo);

                if let Some(error) = self.estado.mensaje_error.as_ref() {
                    if !error.is_empty() {
                        ui.label(
                            egui::RichText::new(error)
                                .heading()
                                .color(egui::Color32::from_rgb(255, 40, 40)),
                        );
                    }
                }

                if ui
                    .add_sized([350., 40.], egui::Button::new("Conectar al sistema"))
                    .clicked()
                {
                    Comando::configurar(&self.enviar_comando, self.configuracion.clone());
                }
            });
    }
}

impl eframe::App for Aplicacion {
    /// Lo que ocurre cada vez que actualizamos
    fn update(&mut self, contexto: &egui::Context, _frame: &mut eframe::Frame) {
        let frame: egui::Frame = egui::Frame {
            fill: contexto.style().visuals.panel_fill,
            ..Default::default()
        };

        // Intentar recibir estado actualizado del sistema
        if let Ok(estado) = self.recibir_estado.try_recv() {
            self.estado = estado;
        }

        egui::CentralPanel::default()
            .frame(frame)
            .show(contexto, |ui| {
                if !self.estado.conectado {
                    self.mostrar_autenticacion(ui);
                } else {
                    self.mostrar_aplicacion(ui);
                }

                egui::Context::request_repaint(contexto)
            });
    }
}
