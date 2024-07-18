/// Comandos que podemos pedirle al sistema de cámaras vía terminal.
#[derive(Debug)]
pub enum Comando {
    Conectar(u64, f64, f64, f64),
    ConectarSinId(f64, f64, f64),
    Desconectar(u64),
    ListarCamaras,
    Camara(u64),
    ModificarUbicacion(u64, f64, f64),
    ModificarRango(u64, f64),
    Ayuda,
    Actualizar,
}
