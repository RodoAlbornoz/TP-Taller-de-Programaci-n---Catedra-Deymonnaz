#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordenadas {
    pub latitud: f64,
    pub longitud: f64,
}

impl Coordenadas {
    // En metros
    pub fn distancia(&self, otras_coordenadas: &Self) -> f64 {
        let d_latitud: f64 = (self.latitud - otras_coordenadas.latitud).to_radians();
        let d_longitud: f64 = (self.longitud - otras_coordenadas.longitud).to_radians();
        let latitud1: f64 = self.latitud.to_radians();
        let latitud2: f64 = otras_coordenadas.latitud.to_radians();

        let a: f64 = (d_latitud / 2.).sin().powi(2)
            + (d_longitud / 2.).sin().powi(2) * latitud1.cos() * latitud2.cos();
        let c: f64 = 2. * a.sqrt().asin();

        6_371_000. * c
    }

    /// Devuelve un vector de coordenadas interpoladas entre la coordenada de comienzo y la
    /// coordenada destino. Interpolar se refiere a que se calculan puntos intermedios entre
    /// las dos coordenadas dadas. El número de pasos indica cuántos puntos intermedios se calculan.
    pub fn interpolar(&self, coordenadas_destino: &Self, pasos: usize) -> Vec<Self> {
        if self == coordenadas_destino {
            return vec![*self];
        }
        let mut puntos_interpolados: Vec<Coordenadas> = Vec::new();

        for i in 0..=pasos {
            let t: f64 = i as f64 / pasos as f64;
            let latitud: f64 = self.latitud + (coordenadas_destino.latitud - self.latitud) * t;
            let longitud: f64 = self.longitud + (coordenadas_destino.longitud - self.longitud) * t;
            puntos_interpolados.push(Self::a_partir_de_latitud_longitud(latitud, longitud));
        }

        puntos_interpolados
    }

    pub fn a_partir_de_latitud_longitud(latitud: f64, longitud: f64) -> Self {
        Coordenadas { latitud, longitud }
    }
}

#[cfg(test)]
mod tests {
    use crate::coordenadas::Coordenadas;

    #[test]
    fn distancia_entre_obelisco_y_luna_park() {
        let obelisco = Coordenadas::a_partir_de_latitud_longitud(-34.6037, -58.3816);
        let luna_park = Coordenadas::a_partir_de_latitud_longitud(-34.6020, -58.3689);

        let distancia = obelisco.distancia(&luna_park);

        assert!(distancia > 1170.);
        assert!(distancia < 1190.);
    }

    #[test]
    fn interpolacion_entre_obelisco_y_luna_park() {
        let obelisco = Coordenadas::a_partir_de_latitud_longitud(-58.3816, -34.6037);
        let luna_park = Coordenadas::a_partir_de_latitud_longitud(-58.3689, -34.6020);

        let interpolacion = obelisco.interpolar(&luna_park, 10);

        assert_eq!(interpolacion.len(), 11);
        assert_eq!(interpolacion[0], obelisco);
        assert_eq!(interpolacion[10], luna_park);
    }
}
