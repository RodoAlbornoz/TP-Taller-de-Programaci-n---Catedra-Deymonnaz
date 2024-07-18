pub mod accion_camara;
pub mod accion_dron;
pub mod accion_incidente;
pub mod listar;

use {accion_camara::AccionCamara, accion_dron::AccionDron, accion_incidente::AccionIncidente};
pub enum Accion {
    Incidente(AccionIncidente),
    Camara(AccionCamara),
    Dron(AccionDron),
}
