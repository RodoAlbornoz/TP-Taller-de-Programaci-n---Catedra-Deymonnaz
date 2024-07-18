use std::hash::Hash;

#[derive(Debug, Clone)]
enum Segmento {
    Texto(String),
    Asteriso,
}

/// Meter dentro de un hashMap
#[derive(Debug, Clone)]
pub struct Topico {
    patron: Vec<Segmento>,
    exacto: bool,
}

impl Topico {
    pub fn new(patron: String) -> Result<Self, String> {
        let mut segmentos = Vec::new();
        let mut exacto = true;

        for str in patron.split('.') {
            if !exacto {
                return Err("Patrón no válido".to_string());
            }

            if str.eq("*") {
                segmentos.push(Segmento::Asteriso);
            } else if str.eq(">") {
                exacto = false;
            } else {
                segmentos.push(Segmento::Texto(str.to_string()));
            }
        }

        Ok(Self {
            patron: segmentos,
            exacto,
        })
    }

    pub fn test(&self, subject: &str) -> bool {
        let segmentos = subject.split('.').collect::<Vec<&str>>();
        if self.patron.len() > segmentos.len() {
            return false;
        }

        if self.exacto && segmentos.len() != self.patron.len() {
            return false;
        }

        for (i, segmento) in segmentos.iter().enumerate() {
            if i >= self.patron.len() {
                return true;
            }

            let segmento_patron = &self.patron[i];

            if let Segmento::Texto(t) = segmento_patron {
                if !t.eq(segmento) {
                    return false;
                }
            }
        }

        true
    }

    pub fn a_texto(&self) -> String {
        let mut s = String::new();
        for segmento in &self.patron {
            match segmento {
                Segmento::Texto(t) => s.push_str(t),
                Segmento::Asteriso => s.push('*'),
            }
            s.push('.');
        }
        if self.exacto {
            s.pop();
        } else {
            s.push('>');
        }
        s
    }
}

impl Hash for Topico {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.a_texto().hash(state)
    }
}

impl PartialEq for Topico {
    fn eq(&self, other: &Self) -> bool {
        self.a_texto().eq(&other.a_texto())
    }
}

impl Eq for Topico {}
