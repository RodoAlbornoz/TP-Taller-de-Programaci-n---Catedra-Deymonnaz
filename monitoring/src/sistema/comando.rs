use std::sync::mpsc::Sender;

use lib::{configuracion::Configuracion, incidente::Incidente};

/// Comandos que se pueden enviar al hilo de la lógica
pub enum Comando {
    Configurar(Configuracion),
    Desconectar,
    NuevoIncidente(Incidente),
    ModificarIncidente(Incidente),
    IncidenteFinalizado(u64),
    ConectarCamara(f64, f64, f64),
    DesconectarCamara(u64),
    CamaraNuevaUbicacion(u64, f64, f64),
    CamaraNuevoRango(u64, f64),
}

impl Comando {
    /// Envía un comando al hilo de la lógica
    fn enviar(canal: &Sender<Comando>, comando: Self) {
        let _ = canal.send(comando);
    }

    /// Envía un comando de nuevo incidente al hilo de la lógica
    pub fn nuevo_incidente(canal: &Sender<Comando>, incidente: Incidente) {
        Self::enviar(canal, Comando::NuevoIncidente(incidente));
    }

    pub fn modificar_incidente(canal: &Sender<Comando>, incidente: Incidente) {
        Self::enviar(canal, Comando::ModificarIncidente(incidente));
    }

    /// Envía un comando de incidente finalizado al hilo de la lógica
    pub fn incidente_finalizado(canal: &Sender<Comando>, id: u64) {
        Self::enviar(canal, Comando::IncidenteFinalizado(id));
    }

    pub fn camara_nueva_ubicacion(canal: &Sender<Comando>, id: u64, latitud: f64, longitud: f64) {
        Self::enviar(canal, Comando::CamaraNuevaUbicacion(id, latitud, longitud));
    }

    pub fn camara_nuevo_rango(canal: &Sender<Comando>, id: u64, rango: f64) {
        Self::enviar(canal, Comando::CamaraNuevoRango(id, rango));
    }

    pub fn conectar_camara(canal: &Sender<Comando>, latitud: f64, longitud: f64, rango: f64) {
        Self::enviar(canal, Comando::ConectarCamara(latitud, longitud, rango));
    }

    pub fn desconectar_camara(canal: &Sender<Comando>, id: u64) {
        Self::enviar(canal, Comando::DesconectarCamara(id));
    }

    pub fn configurar(canal: &Sender<Comando>, configuracion: Configuracion) {
        Self::enviar(canal, Comando::Configurar(configuracion));
    }

    pub fn desconectar(canal: &Sender<Comando>) {
        Self::enviar(canal, Comando::Desconectar);
    }
}
