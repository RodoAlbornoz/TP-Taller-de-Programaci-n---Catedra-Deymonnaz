#[derive(Debug, Clone)]
pub enum Estado {
    EnEspera,
    AtendiendoIncidente,
    YendoAIncidente,
    EsperandoApoyo,
    VolviendoAAreaDeOperacion,
    YendoACentral,
    Recargando,
}

impl Estado {
    pub fn estado_a_str(&self) -> &str {
        match self {
            Estado::EnEspera => "En espera",
            Estado::AtendiendoIncidente => "Atendiendo incidente",
            Estado::YendoACentral => "Yendo a central",
            Estado::Recargando => "Recargando",
            Estado::VolviendoAAreaDeOperacion => "Volviendo a area de operacion",
            Estado::YendoAIncidente => "Yendo a incidente",
            Estado::EsperandoApoyo => "Esperando apoyo",
        }
    }
}
