use std::{thread, time::Duration};

#[derive(Debug, Clone)]
pub struct Bateria {
    /// La bateria actual se mide en segundos restantes para terminar de agotarse
    pub duracion_actual: u64,
    /// La bateria total es la duracion total de la bateria en segundos,
    /// desde que es creada hasta que se agota
    pub duracion_total: u64,
    /// Duracion minima de bateria para que el dron pueda seguir operando
    /// y volver a la central para recargarse
    pub duracion_minima: u64,
    /// Tiempo que tarda en recargarse la bateria del dron
    pub tiempo_recarga: u64,
}

impl Bateria {
    pub fn new(
        duracion_actual: u64,
        duracion_total: u64,
        duracion_minima: u64,
        tiempo_recarga: u64,
    ) -> Self {
        Bateria {
            duracion_actual,
            duracion_total,
            duracion_minima,
            tiempo_recarga,
        }
    }

    pub fn recargar(&mut self) {
        thread::sleep(Duration::from_secs(self.tiempo_recarga));
        self.duracion_actual = self.duracion_total;
    }

    pub fn descargar(&mut self) {
        // 1 segundo menos de bateria
        self.duracion_actual -= 1;
        println!(
            "Al dron le quedan {} segundos de bateria\n",
            self.duracion_actual
        );
    }

    pub fn nivel_minimo(&self) -> bool {
        self.duracion_actual <= self.duracion_minima
    }

    pub fn esta_agotada(&self) -> bool {
        self.duracion_actual == 0
    }
}
