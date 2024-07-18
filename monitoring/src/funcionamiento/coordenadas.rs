use walkers::{Position, Projector};

/// Convierte grados a pixeles
pub fn grados_a_pixeles(posicion: &Position, proyector: &Projector) -> f32 {
    let p2: Position = Position::from_lat_lon(posicion.lat(), posicion.lon() + 1.);

    let p1_en_pantalla: egui::Pos2 = proyector.project(posicion.to_owned()).to_pos2();
    let p2_en_pantalla: egui::Pos2 = proyector.project(p2).to_pos2();

    p2_en_pantalla.x - p1_en_pantalla.x
}

/// Convierte metros a pixeles en el mapa
pub fn metros_a_pixeles_en_mapa(posicion: &Position, proyector: &Projector) -> f64 {
    let pos2: Position = Position::from_lat_lon(posicion.lat(), posicion.lon() + 1.);

    let metros_en_un_grado: f64 = distancia_coordenadas(posicion, &pos2);

    let grados_en_un_metro: f64 = 1.0 / metros_en_un_grado;

    let pixeles_por_grado: f32 = grados_a_pixeles(posicion, proyector);

    (pixeles_por_grado as f64) * grados_en_un_metro
}

/// Calcula la distancia entre dos coordenadas
pub fn distancia_coordenadas(pos1: &Position, pos2: &Position) -> f64 {
    let latitud1: f64 = pos1.lat();
    let longitud1: f64 = pos1.lon();

    let latitud2: f64 = pos2.lat();
    let longitud2: f64 = pos2.lon();

    let d_latitud: f64 = (latitud2 - latitud1).to_radians();
    let d_longitud: f64 = (longitud2 - longitud1).to_radians();

    let a: f64 = (d_latitud / 2.0).sin() * (d_latitud / 2.0).sin()
        + latitud1.to_radians().cos()
            * latitud2.to_radians().cos()
            * (d_longitud / 2.0).sin()
            * (d_longitud / 2.0).sin();

    let c: f64 = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    6_371_000.0 * c
}

/// Devuelve un vector de posiciones interpoladas entre la posicion de comienzo y la
/// posicion destino. Interpolar se refiere a que se calculan puntos intermedios entre
/// las dos posiciones dadas. El número de pasos indica cuántos puntos intermedios se calculan.
pub fn interpolar(comienzo: &Position, destino: &Position, pasos: usize) -> Vec<Position> {
    let mut puntos_interpolados = Vec::with_capacity(pasos);

    for i in 0..=pasos {
        let t = i as f64 / pasos as f64;
        let latitud = comienzo.lat() + (destino.lat() - comienzo.lat()) * t;
        let longitud = comienzo.lon() + (destino.lon() - comienzo.lon()) * t;
        puntos_interpolados.push(Position::from_lat_lon(latitud, longitud));
    }

    puntos_interpolados
}

#[cfg(test)]
mod tests {
    use walkers::Position;

    use crate::funcionamiento::coordenadas::{distancia_coordenadas, interpolar};

    #[test]
    fn distancia_entre_obelisco_y_luna_park() {
        let obelisco = Position::from_lon_lat(-58.3816, -34.6037);
        let luna_park = Position::from_lon_lat(-58.3689, -34.6020);

        let distancia = distancia_coordenadas(&obelisco, &luna_park);

        assert!(distancia > 1170.);
        assert!(distancia < 1190.);
    }

    #[test]
    fn interpolacion_entre_obelisco_y_luna_park() {
        let obelisco = Position::from_lon_lat(-58.3816, -34.6037);
        let luna_park = Position::from_lon_lat(-58.3689, -34.6020);

        let interpolacion = interpolar(&obelisco, &luna_park, 10);

        assert_eq!(interpolacion.len(), 11);
        assert_eq!(interpolacion[0], obelisco);
        assert_eq!(interpolacion[10], luna_park);
    }
}
