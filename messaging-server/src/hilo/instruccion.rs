use crate::{
    conexion::id::IdConexion,
    publicacion::Publicacion,
    suscripciones::{id::IdSuscripcion, suscripcion::Suscripcion},
};

/// Las instrucciones se envian entre "threads" o hilos
#[derive(Clone, Debug)]
pub enum Instruccion {
    /// Añadir una suscripción
    Suscribir(Suscripcion),
    /// Eliminar una suscripción
    Desuscribir(IdConexion, IdSuscripcion),
    /// Publicar, excepto suscripciones de queue group
    Publicar(Publicacion),
    /// Enviar una publicación a una suscripción exacta
    PublicarExacto(Suscripcion, Publicacion),
    /// Este comando se utiliza para poner en cola una nueva publicación
    /// generada por un cliente y enviada al propio thread, esto se hace para evitar
    /// Que el servidor envie la publicación antes de que se genere la suscripcion
    NuevaPublicacion(Publicacion),
}
