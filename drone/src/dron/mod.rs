mod bateria;
pub mod central;
pub mod desplazamiento;
mod estado;

use lib::{
    configuracion::Configuracion,
    coordenadas::Coordenadas,
    csv::{csv_encodear_linea, csv_parsear_linea},
    incidente::Incidente,
    serializables::{deserializar_vec, error::DeserializationError, Serializable},
};

use std::{
    cmp::min,
    collections::HashMap,
    fs::OpenOptions,
    io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write},
    process, thread,
    time::Duration,
    vec::IntoIter,
};

use {
    bateria::Bateria,
    central::Central,
    desplazamiento::Desplazamiento,
    estado::Estado,
    messaging_client::cliente::{suscripcion::Suscripcion, Cliente},
};

pub const NO_ATENDIENDO: u64 = u64::MAX;

#[derive(Debug, Clone)]
pub struct Dron {
    pub id: u64,
    pub desplazamiento: Desplazamiento,
    pub central: Central,
    pub bateria: Bateria,
    /// Incidentes dentro del area de operacion
    incidentes_primarios: HashMap<u64, Incidente>,
    /// Incidentes fuera del area de operacion
    incidentes_secundarios: HashMap<u64, Incidente>,
    otros_drones: HashMap<u64, Dron>,
    pub id_incidente_a_atender: u64,
    pub tiempo_atender_incidente: u64,
    pub estado: Estado,
}

impl Dron {
    pub fn new(configuracion: &Configuracion) -> Dron {
        let id: u64 = configuracion.obtener("id").unwrap_or(0);
        let latitud: f64 = configuracion.obtener("latitud").unwrap_or(0.0);
        let longitud: f64 = configuracion.obtener("longitud").unwrap_or(0.0);
        let rango: f64 = configuracion.obtener("rango").unwrap_or(0.0);
        let velocidad: u64 = configuracion.obtener("velocidad").unwrap_or(0);
        let bateria_duracion_total: u64 = configuracion.obtener("duracion_bateria").unwrap_or(0);
        let bateria_duracion_minima: u64 = configuracion
            .obtener("duracion_bateria_minima")
            .unwrap_or(0);
        let tiempo_recarga: u64 = configuracion.obtener("tiempo_recarga").unwrap_or(0);
        let tiempo_atender_incidente: u64 = configuracion
            .obtener("tiempo_atender_incidente")
            .unwrap_or(0);

        let dron = Dron {
            id,
            desplazamiento: Desplazamiento::new(latitud, longitud, velocidad),
            central: Central::new(id, latitud, longitud, rango),
            bateria: Bateria::new(
                bateria_duracion_total,
                bateria_duracion_total,
                bateria_duracion_minima,
                tiempo_recarga,
            ),
            incidentes_primarios: HashMap::new(),
            incidentes_secundarios: HashMap::new(),
            otros_drones: HashMap::new(),
            id_incidente_a_atender: NO_ATENDIENDO,
            tiempo_atender_incidente,
            estado: Estado::EnEspera,
        };

        println!("Creado dron: {:?}", dron);

        dron
    }

    /// Guarda el estado inicial del dron en la ruta de archivo dada por configuracion o en
    /// drones.csv para que luego pueda volver a cargarse con mas facilidad, solo especificando
    /// el la ruta del archivo y el ID.
    fn guardar_dron(&self, configuracion: &Configuracion) -> io::Result<()> {
        let ruta_archivo_drones: String = configuracion
            .obtener::<String>("drones")
            .unwrap_or("drones.csv".to_string());
        let dron_csv = self.dron_inicial_csv();

        // Abrir el archivo de drones para leer
        let archivo = OpenOptions::new().read(true).open(&ruta_archivo_drones)?;
        let mut lector = BufReader::new(&archivo);

        let mut id_existe = false;
        let mut buffer = Vec::new();
        let mut posicion = 0;
        // Leo línea por línea y verifico si el ID del dron ya existe
        while let Some(Ok(linea)) = lector.by_ref().lines().next() {
            if linea.starts_with(&self.id.to_string()) {
                id_existe = true;
                break;
            }
            posicion += linea.len() + 1; // Actualizo la posicion a sobreescribir si existe el ID
            buffer.push(linea);
        }

        // Abro el archivo nuevamente para escribir
        let mut escritor = OpenOptions::new().write(true).open(&ruta_archivo_drones)?;

        if id_existe {
            // Si el ID ya existia, sobrescribo la linea
            escritor.seek(SeekFrom::Start(posicion as u64))?;
            escritor.write_all(&dron_csv)?;
        } else {
            // Si no existia el ID, escribo al final del archivo
            escritor.seek(SeekFrom::End(0))?;
            escritor.write_all(&dron_csv)?;
        }

        Ok(())
    }

    /// Inicia el proceso de funcionamiento del dron.
    pub fn iniciar(&mut self, configuracion: &Configuracion) -> io::Result<()> {
        self.guardar_dron(configuracion)?;
        let mut cliente: Cliente = self.conectar(configuracion)?;

        let sub_incidentes_nuevos: Suscripcion =
            cliente.suscribirse("incidentes.*.creado", None)?;
        let sub_incidentes_finalizados: Suscripcion =
            cliente.suscribirse("incidentes.*.finalizado", None)?;
        let sub_incidentes = cliente.suscribirse("incidentes", None)?;
        let sub_drones: Suscripcion = cliente.suscribirse("dron.*.informacion", None)?;
        let sub_drones_incidentes_atendidos: Suscripcion =
            cliente.suscribirse("dron.*.id.incidente.atendido", None)?;

        // Inicia el ciclo de eventos del dron
        loop {
            if let Err(e) = self.ciclo(
                &mut cliente,
                &sub_incidentes_nuevos,
                &sub_incidentes_finalizados,
                &sub_incidentes,
                &sub_drones,
                &sub_drones_incidentes_atendidos,
            ) {
                eprintln!("Error en hilo principal: {}", e);
                thread::sleep(Duration::from_secs(1));
            }
        }
    }

    /// Conectar el cliente al servidor.
    fn conectar(&self, configuracion: &Configuracion) -> io::Result<Cliente> {
        let direccion: String = configuracion
            .obtener::<String>("direccion")
            .unwrap_or("127.0.0.1".to_string());

        let puerto: u16 = configuracion.obtener::<u16>("puerto").unwrap_or(4222);
        println!(
            "Conectando al servidor de NATS en {}:{}\n",
            direccion, puerto
        );

        let user: Option<String> = configuracion.obtener::<String>("user");
        let pass: Option<String> = configuracion.obtener::<String>("pass");

        if user.is_some() || pass.is_some() {
            Cliente::conectar_con_user_y_pass(&format!("{}:{}", direccion, puerto), user, pass)
        } else {
            Cliente::conectar(&format!("{}:{}", direccion, puerto))
        }
    }

    /// El bucle de eventos del dron.
    /// Este bucle puede terminar por un error de conexión.
    fn ciclo(
        &mut self,
        cliente: &mut Cliente,
        sub_incidentes_nuevos: &Suscripcion,
        sub_incidentes_finalizados: &Suscripcion,
        sub_incidentes: &Suscripcion,
        sub_drones: &Suscripcion,
        sub_drones_incidentes_atendidos: &Suscripcion,
    ) -> io::Result<()> {
        self.publicar_informacion(cliente)?;

        self.actualizar_bateria(cliente)?;

        self.actualizar_incidentes_activos(sub_incidentes_nuevos, sub_incidentes)?;
        self.actualizar_incidentes_finalizados(
            sub_incidentes_finalizados,
            sub_drones_incidentes_atendidos,
        )?;
        self.actualizar_otros_drones(sub_drones)?;

        if self.ir_hacia_incidente_a_atender(cliente)? {
            self.atender_incidente(cliente, sub_drones)?;
        }

        thread::sleep(Duration::from_secs(1));

        Ok(())
    }

    /// Publica el desplazamiento y estado del dron.
    fn publicar_informacion(&mut self, cliente: &mut Cliente) -> io::Result<()> {
        // Cada dron tiene su topico en el que comunica su posición y estado
        let topico_string: String = format!("dron.{}.informacion", self.id);
        let body: Vec<u8> = self.serializar();

        cliente.publicar(topico_string.as_str(), &body, None)?;

        Ok(())
    }

    /// Publica el ID del incidente atendido.
    fn publicar_incidente_atendido(&mut self, cliente: &mut Cliente) -> io::Result<()> {
        if self.incidente_detectado(self.id_incidente_a_atender) {
            let topico_string: String = format!("dron.{}.id.incidente.atendido", self.id);
            let body: String = self.id_incidente_a_atender.to_string();

            cliente.publicar(topico_string.as_str(), body.as_bytes(), None)?;
        }
        Ok(())
    }

    /// Carga los incidentes activos, o sea, los nuevos y los previos a la creacion del dron.
    fn actualizar_incidentes_activos(
        &mut self,
        sub_incidentes_nuevos: &Suscripcion,
        sub_incidentes: &Suscripcion,
    ) -> io::Result<()> {
        while let Some(mensaje) = sub_incidentes_nuevos.intentar_leer()? {
            match Incidente::deserializar(&mensaje.payload) {
                Ok(incidente) => {
                    self.cargar_incidente(incidente);
                }
                Err(_) => eprintln!("Error al deserializar incidente"),
            }
        }
        while let Some(mensaje) = sub_incidentes.intentar_leer()? {
            let incidentes: Vec<Incidente> = deserializar_vec(&mensaje.payload).unwrap_or_default();
            for incidente in incidentes {
                self.cargar_incidente(incidente);
            }
        }
        Ok(())
    }

    /// Finaliza los incidentes que ya fueron atendidos por drones o finalizados desde el monitoring.
    fn actualizar_incidentes_finalizados(
        &mut self,
        sub_incidentes_finalizados: &Suscripcion,
        sub_drones_incidentes_atendidos: &Suscripcion,
    ) -> io::Result<()> {
        while let Some(mensaje) = sub_incidentes_finalizados.intentar_leer()? {
            match Incidente::deserializar(&mensaje.payload) {
                Ok(incidente) => {
                    println!("Incidente finalizado: {}", &incidente.id);
                    self.finalizar_incidente(incidente.id);
                }
                Err(_) => eprintln!("Error al deserializar incidente"),
            }
        }
        while let Some(mensaje) = sub_drones_incidentes_atendidos.intentar_leer()? {
            let id_incidente: u64 = String::from_utf8(mensaje.payload)
                .unwrap_or_default()
                .parse()
                .unwrap_or_default();
            println!("Incidente atendido por dron: {}", id_incidente);
            self.finalizar_incidente(id_incidente);
        }
        Ok(())
    }

    /// Carga en el dron un incidente.
    fn cargar_incidente(&mut self, incidente: Incidente) {
        if self.desplazamiento.es_alcanzable(
            incidente.latitud,
            incidente.longitud,
            self.bateria.duracion_total as i64,
            self.tiempo_atender_incidente as i64,
        ) {
            if incidente.posicion().distancia(&self.central.coordenadas()) < self.central.rango {
                println!("Incidente dentro de area de operacion: {:?}", incidente);
                self.incidentes_primarios.insert(incidente.id, incidente);
            } else {
                println!("Incidente fuera de area de operacion: {:?}", incidente);
                self.incidentes_secundarios.insert(incidente.id, incidente);
            }
        }
    }

    /// Finaliza el incidente con el ID dado.
    fn finalizar_incidente(&mut self, id_incidente: u64) {
        self.incidentes_primarios.remove(&id_incidente);
        self.incidentes_secundarios.remove(&id_incidente);
    }

    /// Actualiza el dron con la informacion de los demas drones activos.
    fn actualizar_otros_drones(&mut self, sub_drones: &Suscripcion) -> io::Result<()> {
        while let Some(mensaje) = sub_drones.intentar_leer()? {
            match Dron::deserializar(&mensaje.payload) {
                Ok(dron) => {
                    if self.id != dron.id {
                        self.otros_drones.insert(dron.id, dron);
                    }
                }
                Err(_) => eprintln!("Error al deserializar dron"),
            }
        }
        Ok(())
    }

    /// El dron se mueve hacia la latitud y longitud destino. Se asegura de que este dentro
    /// de su alcance maximo, o sea, tener suficiente bateria para ir y volver.
    /// Devuelve true si llego al destino, false si no se pudo mover.
    pub fn mover(
        &mut self,
        cliente: &mut Cliente,
        latitud_destino: f64,
        longitud_destino: f64,
        estado_durante_desplazamiento: Estado,
    ) -> io::Result<bool> {
        if !self.desplazamiento.esta_en_el_alcance_maximo(
            latitud_destino,
            longitud_destino,
            self.bateria.duracion_actual as i64,
            self.tiempo_atender_incidente as i64,
        ) {
            println!("El dron no tiene bateria suficiente para llegar al objetivo\n");
            self.id_incidente_a_atender = NO_ATENDIENDO;
            self.recargar_en_central(cliente)?;
            return Ok(false);
        }

        println!("{}\n", estado_durante_desplazamiento.estado_a_str());
        self.estado = estado_durante_desplazamiento;
        let interpolaciones: Vec<Coordenadas> = self
            .desplazamiento
            .interpolaciones(latitud_destino, longitud_destino);
        for coordenadas in interpolaciones {
            self.desplazamiento
                .actualizar_latitud_longitud(coordenadas.latitud, coordenadas.longitud);
            self.bateria.descargar();

            self.publicar_informacion(cliente)?;
            thread::sleep(Duration::from_secs(1));
        }
        self.estado = Estado::EnEspera;

        Ok(true)
    }

    /// El dron busca el incidente mas cercano y una vez hallado, se desplaza hacia el.
    fn ir_hacia_incidente_a_atender(&mut self, cliente: &mut Cliente) -> io::Result<bool> {
        if self.id_incidente_a_atender == NO_ATENDIENDO {
            if let Some(incidente) = self.incidente_a_atender() {
                let id_incidente_a_atender: u64 = incidente.id;
                if self.cantidad_drones_atendiendo_incidente(id_incidente_a_atender) < 2
                    && self.mover(
                        cliente,
                        incidente.latitud,
                        incidente.longitud,
                        Estado::YendoAIncidente,
                    )?
                {
                    self.id_incidente_a_atender = id_incidente_a_atender;
                    self.publicar_informacion(cliente)?;
                    println!("En incidente\n");
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// Atiende el incidente asignado (se asume que se esta sobre el incidente en esta instancia,
    /// ya que id_incidente_a_atender se asigna solo si se ha completado el desplazamiento).
    fn atender_incidente(
        &mut self,
        cliente: &mut Cliente,
        sub_drones: &Suscripcion,
    ) -> io::Result<()> {
        if !self.incidente_detectado(self.id_incidente_a_atender) {
            self.volver_a_area_de_operacion(cliente)?;
            return Ok(());
        }

        self.estado = Estado::EsperandoApoyo;
        self.publicar_informacion(cliente)?;
        while self.cantidad_drones_atendiendo_incidente(self.id_incidente_a_atender) < 2 {
            self.actualizar_otros_drones(sub_drones)?;
            self.actualizar_bateria(cliente)?;
            // Si luego de actualizar la bateria, el dron llega a la bateria minima para volver,
            // se cancela la atencion del incidente por esta iteracion y vuelve a la central
            if self.bateria.duracion_actual as usize
                <= self
                    .desplazamiento
                    .segundos_hasta_destino(self.central.latitud, self.central.longitud)
            {
                self.volver_a_area_de_operacion(cliente)?;
                return Ok(());
            }
            self.publicar_informacion(cliente)?;
            println!("Esperando apoyo en el incidente\n");
            thread::sleep(Duration::from_secs(1));
        }
        // Al salir del while ya se sabe que hay 2 drones en el incidente, por lo que se puede
        // proceder a atenderlo

        if !self.incidente_detectado(self.id_incidente_a_atender) {
            self.volver_a_area_de_operacion(cliente)?;
            return Ok(());
        }

        self.estado = Estado::AtendiendoIncidente;
        self.publicar_informacion(cliente)?;

        let tiempo_atencion_otro_dron: u64 =
            self.tiempo_atencion_del_otro_dron_en_incidente(self.id_incidente_a_atender);

        // Asumo que el incidente se resolvera en el tiempo de atencion minimo entre ambos drones
        thread::sleep(Duration::from_secs(min(
            self.tiempo_atender_incidente,
            tiempo_atencion_otro_dron,
        )));
        self.publicar_incidente_atendido(cliente)?;
        self.finalizar_incidente(self.id_incidente_a_atender);
        self.id_incidente_a_atender = NO_ATENDIENDO;
        self.publicar_informacion(cliente)?;
        self.actualizar_otros_drones(sub_drones)?;

        self.volver_a_area_de_operacion(cliente)?;
        Ok(())
    }

    /// Aca se asume que ya no se esta atendiendo un incidente en esta iteracion y se vuelve
    /// a la central, que es un punto de referencia del area de operacion.
    fn volver_a_area_de_operacion(&mut self, cliente: &mut Cliente) -> io::Result<()> {
        self.id_incidente_a_atender = NO_ATENDIENDO;
        if !self.en_central() {
            self.mover(
                cliente,
                self.central.latitud,
                self.central.longitud,
                Estado::VolviendoAAreaDeOperacion,
            )?;
        }
        Ok(())
    }

    /// Devuelve la cantidad de drones atendiendo el incidente con el ID dado, es decir,
    /// ya en la posicion del incidente.
    fn cantidad_drones_atendiendo_incidente(&self, id_incidente: u64) -> u64 {
        let mut cantidad: u64 = 0;
        if self.id_incidente_a_atender == id_incidente {
            cantidad += 1;
        }
        cantidad += self
            .otros_drones
            .values()
            .filter(|dron| dron.id_incidente_a_atender == id_incidente)
            .count() as u64;
        println!(
            "Dron {}: Cantidad de drones atendiendo incidente {}: {}\n",
            self.id, id_incidente, cantidad
        );
        cantidad
    }

    /// Devuelve el incidente mas cercano a la posicion actual del dron, priorizando los
    /// incidentes con un solo dron atendiendolos, para brindar apoyo. En caso contrario, los
    /// incidentes dentro del area de operacion, sino los incidentes fuera del area de operacion,
    /// pero dentro del alcance maximo.
    fn incidente_a_atender(&self) -> Option<Incidente> {
        if let Some(incidente) = self.incidente_mas_cercano(&self.incidentes_con_drones_solos()) {
            Some(incidente)
        } else if !self.incidentes_primarios.is_empty() {
            self.incidente_mas_cercano(&self.incidentes_primarios)
        } else if !self.incidentes_secundarios.is_empty() {
            self.incidente_mas_cercano(&self.incidentes_secundarios)
        } else {
            None
        }
    }

    /// Devuelve el incidente mas cercano a la posicion actual del dron, dado un conjunto
    /// de incidentes.
    fn incidente_mas_cercano<'a>(
        &'a self,
        incidentes: &'a HashMap<u64, Incidente>,
    ) -> Option<Incidente> {
        incidentes
            .values()
            .min_by(|incidente_a, incidente_b| {
                self.desplazamiento
                    .coordenadas()
                    .distancia(&Coordenadas::a_partir_de_latitud_longitud(
                        incidente_a.latitud,
                        incidente_a.longitud,
                    ))
                    .partial_cmp(&self.desplazamiento.coordenadas().distancia(
                        &Coordenadas::a_partir_de_latitud_longitud(
                            incidente_b.latitud,
                            incidente_b.longitud,
                        ),
                    ))
                    .unwrap()
            })
            .cloned()
    }

    /// Devuelve un conjunto de los drones que estan atendiendo solos un incidente.
    fn incidentes_con_drones_solos(&self) -> HashMap<u64, Incidente> {
        let mut incidentes_con_drones_solos: HashMap<u64, Incidente> = HashMap::new();
        for (id, incidente) in self.incidentes_primarios.iter() {
            if self.cantidad_drones_atendiendo_incidente(*id) == 1 {
                incidentes_con_drones_solos.insert(*id, incidente.clone());
            }
        }
        for (id, incidente) in self.incidentes_secundarios.iter() {
            if self.cantidad_drones_atendiendo_incidente(*id) == 1 {
                incidentes_con_drones_solos.insert(*id, incidente.clone());
            }
        }
        incidentes_con_drones_solos
    }

    /// Devuelve el tiempo que demora el otro dron en atender un incidente con el ID dado.
    fn tiempo_atencion_del_otro_dron_en_incidente(&self, id_incidente: u64) -> u64 {
        for dron in self.otros_drones.values() {
            if dron.id_incidente_a_atender == id_incidente {
                return dron.tiempo_atender_incidente;
            }
        }
        // Si no hay ningun dron atendiendo el incidente, aunque este caso no deberia darse
        // dado el contexto en el que se usa esta funcion por unica vez
        NO_ATENDIENDO
    }

    /// Actualiza la bateria del dron, si esta en niveles minimos, el dron se desplaza
    /// a la central donde procede a recargarse. Si hubo algun desperfecto y el dron se
    /// queda sin bateria, se apagara en el lugar y dejara de funcionar.
    fn actualizar_bateria(&mut self, cliente: &mut Cliente) -> io::Result<()> {
        if self.bateria.esta_agotada() {
            println!("Bateria agotada, el dron se apagara\n");
            process::exit(1);
        }

        self.bateria.descargar();

        if self.bateria.nivel_minimo() {
            self.recargar_en_central(cliente)?;
        }

        Ok(())
    }

    /// El dron recarga su bateria en la central, si no esta en la central, procede
    /// a desplazarse hacia ella.
    fn recargar_en_central(&mut self, cliente: &mut Cliente) -> io::Result<()> {
        self.id_incidente_a_atender = NO_ATENDIENDO;
        if self.en_central() {
            self.estado = Estado::Recargando;
            println!("Recargando en central\n");
            self.publicar_informacion(cliente)?;

            self.bateria.recargar();
            println!("Bateria recargada");
            self.estado = Estado::EnEspera;
            self.publicar_informacion(cliente)?;
        } else {
            self.estado = Estado::YendoACentral;
            println!("Yendo a central\n");

            self.mover(
                cliente,
                self.central.latitud,
                self.central.longitud,
                Estado::YendoACentral,
            )?;

            println!("En central\n")
        }
        Ok(())
    }

    /// Devuelve true si el dron se encuentra en la central, false en caso contrario.
    fn en_central(&self) -> bool {
        self.desplazamiento.coordenadas() == self.central.coordenadas()
    }

    /// Devuelve true si el dron detecto el incidente con el ID dado, false en caso contrario.
    fn incidente_detectado(&self, id_incidente: u64) -> bool {
        self.incidentes_primarios.contains_key(&id_incidente)
            || self.incidentes_secundarios.contains_key(&id_incidente)
    }

    /// Devuelve el ID del incidente en tipo String para mostrarlo en el monitoring.
    pub fn id_incidente_a_atender_a_string(&self) -> String {
        if self.id_incidente_a_atender == NO_ATENDIENDO {
            "Ninguno".to_string()
        } else {
            self.id_incidente_a_atender.to_string()
        }
    }

    /// Devuelve los datos iniciales del dron en formato CSV para guardarlo en el
    /// archivo dado o en drones.csv.
    fn dron_inicial_csv(&self) -> Vec<u8> {
        let dron_string = format!(
            "{},{},{},{},{},{},{},{},{}\n",
            self.id,
            self.central.latitud,
            self.central.longitud,
            self.central.rango,
            self.desplazamiento.velocidad,
            self.bateria.duracion_total,
            self.bateria.duracion_minima,
            self.bateria.tiempo_recarga,
            self.tiempo_atender_incidente,
        );
        dron_string.into_bytes()
    }
}

impl Serializable for Dron {
    fn serializar(&self) -> Vec<u8> {
        let mut parametros: Vec<String> = Vec::new();
        parametros.push(format!("{}", self.id));
        parametros.push(format!("{}", self.desplazamiento.latitud));
        parametros.push(format!("{}", self.desplazamiento.longitud));
        parametros.push(format!("{}", self.central.latitud));
        parametros.push(format!("{}", self.central.longitud));
        parametros.push(format!("{}", self.central.rango));
        parametros.push(format!("{}", self.desplazamiento.velocidad));
        parametros.push(format!("{}", self.bateria.duracion_actual));
        parametros.push(format!("{}", self.bateria.duracion_total));
        parametros.push(format!("{}", self.bateria.duracion_minima));
        parametros.push(format!("{}", self.bateria.tiempo_recarga));
        parametros.push(format!("{}", self.tiempo_atender_incidente));
        parametros.push(format!("{}", self.id_incidente_a_atender));
        parametros.push(self.estado.estado_a_str().to_string());
        csv_encodear_linea(&parametros).into_bytes()
    }

    fn deserializar(datos: &[u8]) -> Result<Self, lib::serializables::error::DeserializationError>
    where
        Self: Sized,
    {
        let linea: String =
            String::from_utf8(datos.to_vec()).map_err(|_| DeserializationError::InvalidData)?;
        let mut parametros: IntoIter<String> = csv_parsear_linea(linea.as_str()).into_iter();

        let id: u64 = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let latitud: f64 = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let longitud: f64 = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let latitud_central: f64 = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let longitud_central: f64 = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let rango: f64 = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let velocidad: u64 = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let bateria_duracion_actual: u64 = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let bateria_duracion_total: u64 = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let bateria_duracion_minima: u64 = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let tiempo_recarga: u64 = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let tiempo_atender_incidente: u64 = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let id_incidente_a_atender: u64 = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;

        let estado_string: String = parametros
            .next()
            .ok_or(DeserializationError::MissingField)?
            .parse()
            .map_err(|_| DeserializationError::InvalidData)?;
        let estado_op = match estado_string.as_str() {
            "En espera" => Some(Estado::EnEspera),
            "Yendo a central" => Some(Estado::YendoACentral),
            "Recargando" => Some(Estado::Recargando),
            "Volviendo a area de operacion" => Some(Estado::VolviendoAAreaDeOperacion),
            "Yendo a incidente" => Some(Estado::YendoAIncidente),
            "Atendiendo incidente" => Some(Estado::AtendiendoIncidente),
            "Esperando apoyo" => Some(Estado::EsperandoApoyo),
            _ => None,
        };

        if let Some(estado) = estado_op {
            Ok(Dron {
                id,
                desplazamiento: Desplazamiento::new(latitud, longitud, velocidad),
                central: Central::new(id, latitud_central, longitud_central, rango),
                bateria: Bateria::new(
                    bateria_duracion_actual,
                    bateria_duracion_total,
                    bateria_duracion_minima,
                    tiempo_recarga,
                ),
                incidentes_primarios: HashMap::new(),
                incidentes_secundarios: HashMap::new(),
                otros_drones: HashMap::new(),
                id_incidente_a_atender,
                tiempo_atender_incidente,
                estado,
            })
        } else {
            Err(DeserializationError::InvalidData)
        }
    }
}
