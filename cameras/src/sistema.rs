use std::{
    borrow::Cow,
    fs,
    io::{self, Error, ErrorKind},
    path::Path,
    sync::mpsc::{Receiver, Sender},
    thread,
    time::Duration,
};

use lib::{
    camara::Camara,
    configuracion::Configuracion,
    incidente::Incidente,
    serializables::{
        deserializar_vec,
        guardar::{cargar_serializable, guardar_serializable},
        serializar_vec, Serializable,
    },
};
use messaging_client::cliente::{suscripcion::Suscripcion, Cliente};

use crate::{
    estado::Estado,
    interfaz::{comando::Comando, interpretar_comando, respuesta::Respuesta},
};

pub struct Sistema {
    pub estado: Estado,
    pub configuracion: Configuracion,
    enviar_respuesta: Sender<Respuesta>,
    recibir_comandos: Receiver<Comando>,
}

impl Sistema {
    pub fn new(
        estado: Estado,
        configuracion: Configuracion,
        enviar_respuesta: Sender<Respuesta>,
        recibir_comandos: Receiver<Comando>,
    ) -> Self {
        Self {
            estado,
            configuracion,
            enviar_respuesta,
            recibir_comandos,
        }
    }

    /// Inicia el bucle infinito del sistema
    ///
    /// Está función se encarga de reintentar la ejecución del sistema en caso de error.
    pub fn iniciar(&mut self) -> io::Result<()> {
        self.cargar_camaras()?;

        loop {
            if let Err(e) = self.inicio() {
                eprintln!("Error en hilo principal: {}", e);
                thread::sleep(Duration::from_secs(1));
            }
        }
    }

    /// Inicia el bucle de eventos del sistema
    ///
    /// Este bucle puede terminar por un error de conexión
    fn inicio(&mut self) -> io::Result<()> {
        // Conectar el cliente al servidor de NATS
        let mut cliente = self.conectar()?;

        // Publicar al servidor de NATS el estado de todas las cámaras
        self.publicar_y_guardar_estado_general(&cliente)?;

        let sub_nuevos_incidentes: Suscripcion =
            cliente.suscribirse("incidentes.*.creado", None)?;
        let sub_incidentes_finalizados: Suscripcion =
            cliente.suscribirse("incidentes.*.finalizado", None)?;
        let sub_comandos_remotos: Suscripcion = cliente.suscribirse("comandos.camaras", None)?;

        let sub_incidentes: Suscripcion = cliente.suscribirse("incidentes", None)?;

        self.solicitar_actualizacion_incidentes(&cliente)?;

        loop {
            self.ciclo(
                &cliente,
                &sub_nuevos_incidentes,
                &sub_incidentes_finalizados,
                &sub_comandos_remotos,
                &sub_incidentes,
            )?;
        }
    }

    /// Conectar el cliente
    fn conectar(&self) -> io::Result<Cliente> {
        let direccion: String = self
            .configuracion
            .obtener::<String>("direccion")
            .unwrap_or("127.0.0.1".to_string());

        let puerto: u16 = self.configuracion.obtener::<u16>("puerto").unwrap_or(4222);
        println!("Conectando al servidor de NATS en {}:{}", direccion, puerto);

        let user: Option<String> = self.configuracion.obtener::<String>("user");
        let pass: Option<String> = self.configuracion.obtener::<String>("pass");

        if user.is_some() || pass.is_some() {
            Cliente::conectar_con_user_y_pass(&format!("{}:{}", direccion, puerto), user, pass)
        } else {
            Cliente::conectar(&format!("{}:{}", direccion, puerto))
        }
    }

    /// Publica al servidor el estado de todas las camaras en el topico "camaras". Toma un
    /// vector de camaras, las guarda serializandolas y escribiendolas a un csv, y publica
    /// el vector serializado.
    fn publicar_y_guardar_estado_general(&mut self, cliente: &Cliente) -> io::Result<()> {
        let camaras: Vec<Camara> = self.estado.camaras().into_iter().cloned().collect();
        let bytes: Vec<u8> = serializar_vec(&camaras);
        self.guardar_camaras()?;
        cliente.publicar("camaras", &bytes, None)
    }

    fn solicitar_actualizacion_incidentes(&self, cliente: &Cliente) -> io::Result<()> {
        cliente.publicar("comandos.monitoreo", b"actualizar", None)
    }

    fn guardar_camaras(&self) -> io::Result<()> {
        let ruta_archivo_camaras: String = self
            .configuracion
            .obtener::<String>("camaras")
            .unwrap_or("camaras.csv".to_string());

        let camaras: Vec<Camara> = self.estado.camaras().into_iter().cloned().collect();
        guardar_serializable(&camaras, &ruta_archivo_camaras)
    }

    fn cargar_camaras(&mut self) -> io::Result<()> {
        let ruta_archivo_camaras: String = self
            .configuracion
            .obtener::<String>("camaras")
            .unwrap_or("camaras.csv".to_string());

        let existe: bool = Path::new(&ruta_archivo_camaras).exists();

        // Si no existe el nombre del archivo de camaras, se crea
        if !existe {
            fs::File::create(&ruta_archivo_camaras)?;
        }

        let mut camaras: Vec<Camara> = cargar_serializable(&ruta_archivo_camaras)?;

        for mut camara in camaras.drain(..) {
            camara.incidentes_primarios.clear();
            camara.incidentes_secundarios.clear();
            self.estado.conectar_camara(camara);
        }

        Ok(())
    }

    /// Ciclo de eventos del sistema
    fn ciclo(
        &mut self,
        cliente: &Cliente,
        sub_nuevos_incidentes: &Suscripcion,
        sub_incidentes_finalizados: &Suscripcion,
        sub_comandos: &Suscripcion,
        sub_incidentes: &Suscripcion,
    ) -> io::Result<()> {
        self.leer_incidentes(
            cliente,
            sub_nuevos_incidentes,
            sub_incidentes_finalizados,
            sub_incidentes,
        )?;
        self.leer_comandos(cliente)?;
        self.leer_comandos_remotos(cliente, sub_comandos)?;

        thread::sleep(Duration::from_millis(5));

        Ok(())
    }

    /// Lee incidentes desde el servidor de NATS
    /// y los procesa. Cambia el estado del sistema
    fn leer_incidentes(
        &mut self,
        cliente: &Cliente,
        sub_nuevos_incidentes: &Suscripcion,
        sub_incidentes_finalizados: &Suscripcion,
        sub_incidentes: &Suscripcion,
    ) -> io::Result<()> {
        let mut enviar_actualizacion: bool = false;

        while let Some(mensaje) = sub_nuevos_incidentes.intentar_leer()? {
            match Incidente::deserializar(&mensaje.payload) {
                Ok(incidente) => {
                    self.estado.cargar_incidente(incidente);
                    enviar_actualizacion = true;
                }
                Err(_) => eprintln!("Error al deserializar incidente"),
            }
        }

        while let Some(mensaje) = sub_incidentes_finalizados.intentar_leer()? {
            match Incidente::deserializar(&mensaje.payload) {
                Ok(incidente) => {
                    self.estado.finalizar_incidente(incidente.id);
                    enviar_actualizacion = true;
                }
                Err(_) => eprintln!("Error al deserializar incidente"),
            };
        }

        while let Some(mensaje) = sub_incidentes.intentar_leer()? {
            let incidentes: Vec<Incidente> = deserializar_vec(&mensaje.payload).unwrap_or_default();

            self.estado.finalizar_todos_los_incidentes();

            println!("Actualizados {} incidentes", incidentes.len());

            for incidente in incidentes {
                self.estado.cargar_incidente(incidente);
            }

            enviar_actualizacion = true;
        }

        if enviar_actualizacion {
            self.publicar_y_guardar_estado_general(cliente)?;
        }

        Ok(())
    }

    /// Lee comandos desde la interfaz y los procesa
    fn leer_comandos(&mut self, cliente: &Cliente) -> io::Result<()> {
        while let Ok(comando) = self.recibir_comandos.try_recv() {
            self.matchear_comandos(cliente, comando)?;
        }

        Ok(())
    }

    fn matchear_comandos(&mut self, cliente: &Cliente, comando: Comando) -> io::Result<()> {
        match comando {
            Comando::Conectar(id, latitud, longitud, rango) => {
                self.comando_conectar_camara(cliente, id, latitud, longitud, rango)?
            }
            Comando::ConectarSinId(latitud, longitud, rango) => {
                let id: u64 = self.buscar_id_camara();
                self.comando_conectar_camara(cliente, id, latitud, longitud, rango)?
            }
            Comando::Desconectar(id) => self.comando_desconectar_camara(cliente, id)?,
            Comando::ListarCamaras => self.comando_listar_camaras()?,
            Comando::ModificarRango(id, rango) => {
                self.comando_modificar_rango(cliente, id, rango)?
            }
            Comando::ModificarUbicacion(id, latitud, longitud) => {
                self.comando_modificar_ubicacion(cliente, id, latitud, longitud)?
            }
            Comando::Camara(id) => self.comando_mostrar_camara(id)?,
            Comando::Ayuda => self.comando_ayuda()?,
            Comando::Actualizar => self.publicar_y_guardar_estado_general(cliente)?,
        }
        Ok(())
    }

    fn buscar_id_camara(&self) -> u64 {
        let mut max_id = 1;
        for camara in self.estado.camaras() {
            if camara.id > max_id {
                max_id = camara.id;
            }
        }
        max_id + 1
    }

    fn leer_comandos_remotos(
        &mut self,
        cliente: &Cliente,
        sub_comandos_remotos: &Suscripcion,
    ) -> io::Result<()> {
        while let Some(mensaje) = sub_comandos_remotos.intentar_leer()? {
            let mensaje_texto: Cow<'_, str> = String::from_utf8_lossy(&mensaje.payload);

            if let Some(comando) = interpretar_comando(&mensaje_texto) {
                self.matchear_comandos(cliente, comando)?;
            }
        }

        Ok(())
    }

    fn comando_conectar_camara(
        &mut self,
        cliente: &Cliente,
        id: u64,
        latitud: f64,
        longitud: f64,
        rango: f64,
    ) -> io::Result<()> {
        if self.estado.camara(id).is_some() {
            return self.responder(Respuesta::Error(
                "Ya existe una cámara con ese ID".to_string(),
            ));
        }
        let camara: Camara = Camara::new(id, latitud, longitud, rango);
        self.estado.conectar_camara(camara);
        self.publicar_y_guardar_estado_general(cliente)?;
        self.responder_ok()
    }

    fn comando_desconectar_camara(&mut self, cliente: &Cliente, id: u64) -> io::Result<()> {
        if self.estado.desconectar_camara(id).is_some() {
            self.publicar_y_guardar_estado_general(cliente)?;
            self.responder_ok()
        } else {
            self.responder(Respuesta::Error(
                "No existe una cámara con ese ID".to_string(),
            ))
        }
    }

    fn comando_listar_camaras(&mut self) -> io::Result<()> {
        let camaras: Vec<Camara> = self.estado.camaras().into_iter().cloned().collect();

        if camaras.is_empty() {
            self.responder(Respuesta::Error("No hay cámaras conectadas".to_string()))
        } else {
            self.responder(Respuesta::Camaras(camaras))
        }
    }

    fn comando_modificar_rango(
        &mut self,
        cliente: &Cliente,
        id: u64,
        rango: f64,
    ) -> io::Result<()> {
        if self.estado.camara(id).is_none() {
            return self.responder(Respuesta::Error(
                "No existe una cámara con ese ID".to_string(),
            ));
        }

        self.estado.modificar_rango_camara(id, rango);
        self.publicar_y_guardar_estado_general(cliente)?;
        self.responder_ok()
    }

    fn comando_modificar_ubicacion(
        &mut self,
        cliente: &Cliente,
        id: u64,
        latitud: f64,
        longitud: f64,
    ) -> io::Result<()> {
        if self.estado.camara(id).is_none() {
            return self.responder(Respuesta::Error(
                "No existe una cámara con ese ID".to_string(),
            ));
        }

        self.estado
            .modificar_ubicacion_camara(id, latitud, longitud);
        self.publicar_y_guardar_estado_general(cliente)?;
        self.responder_ok()
    }

    fn comando_mostrar_camara(&mut self, id: u64) -> io::Result<()> {
        if let Some(camara) = self.estado.camara(id) {
            self.responder(Respuesta::Camara(camara.clone()))
        } else {
            self.responder(Respuesta::Error(
                "No existe una cámara con ese ID".to_string(),
            ))
        }
    }

    fn comando_ayuda(&mut self) -> io::Result<()> {
        self.responder(Respuesta::Ayuda)
    }

    fn responder_ok(&self) -> io::Result<()> {
        self.responder(Respuesta::Ok)
    }

    fn responder(&self, respuesta: Respuesta) -> io::Result<()> {
        self.enviar_respuesta
            .send(respuesta)
            .map_err(|e| Error::new(ErrorKind::Other, e))
    }
}
