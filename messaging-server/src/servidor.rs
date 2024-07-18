use std::{
    collections::HashMap,
    io,
    net::TcpListener,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use lib::configuracion::Configuracion;

use crate::{
    conexion::id::IdConexion,
    cuenta::Cuenta,
    hilo::{id::IdHilo, instruccion::Instruccion},
    registrador::Registrador,
};

use super::{conexion::Conexion, hilo::Hilo};

type InfoHilo = (Sender<(IdConexion, Conexion)>, JoinHandle<()>);

pub struct Servidor {
    pub configuracion: Configuracion,
    hilos: Vec<InfoHilo>,
    // Cada conexión que se genera hay que asignarla a un hilo. Con esto
    // determino a que hilo se lo doy. Si ponemos IdHilo no sirve como indice
    // para Vec, pero si se puede convertir usize a IdHilo
    proximo_id_hilo: usize,
    // Cada id tiene que ser único por cada conexion. Se incrementa cada vez
    // que se crea una nueva conexion
    ultimo_id_conexion: IdConexion,
    registrador: Registrador,
    pub cuentas: Option<Arc<Vec<Cuenta>>>,
}

impl Servidor {
    pub fn desde_configuracion(configuracion: Configuracion) -> Servidor {
        // La cantidad es la cantidad de hilos que se van a crear
        // Vector con los canales para enviar nuevas conexiones y handle de los threads
        let mut hilos: Vec<InfoHilo> = Vec::new();

        // Puntas emisoras de los canales para enviar mensajes a los hilos
        let mut canales_enviar: Vec<Sender<Instruccion>> = Vec::new();
        // Puntas receptoras de los canales para recibir mensajes de los hilos
        let mut canales_recibir: Vec<Receiver<Instruccion>> = Vec::new();

        // `logger`
        let registrador: Registrador = Registrador::new();

        let cantidad: usize = configuracion.obtener::<usize>("hilos").unwrap_or(4);

        // Creamos los canales para enviar y recibir instrucciones entre los hilos
        for _ in 0..cantidad {
            let (tx, rx) = channel();
            canales_enviar.push(tx);
            canales_recibir.push(rx);
        }

        // Para cada punta receptora en canales_recibir, se insertan las
        // puntas emisoras de los canales en canales_a_enviar_mensajes que
        // tiene las puntas emisoras a cada hilo para enviar instrucciones
        // a ellos
        for (indice_hilo, rx) in canales_recibir.drain(..).enumerate() {
            // HashMap con las puntas emisoras a cada hilo para enviar instrucciones a los mismos
            let mut canales_a_enviar_mensajes: HashMap<u64, Sender<Instruccion>> = HashMap::new();

            // Insertamos las puntas emisoras de los canales en el HashMap
            for (id_canal_a_enviar, tx) in canales_enviar.iter().enumerate() {
                let id: u64 = id_canal_a_enviar as IdHilo;
                // El id es el id del hilo. Yo quiero mandarle mensaje a todos los hilos.
                // A cada id, le asigno un emisor a ese hilo. (id 2, le asigno un emisor al hilo 2)
                canales_a_enviar_mensajes.insert(id, tx.clone());
            }

            // Obtengo el id del hilo
            let id_hilo: u64 = indice_hilo as IdHilo; // Id del hilo actual. Suponiendo cronologia; 1, 2...

            // Creamos el canal para enviar nuevas conexiones al hilo
            let (tx_conexiones, rx_conexiones) = channel();
            // Creamos el registrador para el hilo
            let mut registrador: Registrador = registrador.clone();
            // Establecemos el hilo actual para el registrador
            registrador.establecer_hilo(id_hilo);
            // Creamos el hilo
            let hilo: Hilo = Hilo::new(
                id_hilo,
                rx_conexiones,
                canales_a_enviar_mensajes,
                rx,
                registrador,
            );

            // Iniciamos el thread del hilo
            let handle: JoinHandle<()> = Hilo::iniciar(hilo);
            // Tx_conexciones es por donde le van a asignar conexiones al hilo y el handle del hilo
            hilos.push((tx_conexiones, handle));
        }

        Servidor {
            hilos,
            configuracion,
            proximo_id_hilo: 0,
            ultimo_id_conexion: 0,
            registrador,
            cuentas: None,
        }
    }

    pub fn cargar_cuentas(&mut self, ruta_archivo_cuentas: String) -> io::Result<()> {
        let cuentas: Vec<Cuenta> = Cuenta::cargar(&ruta_archivo_cuentas)?;
        self.cuentas = Some(Arc::new(cuentas));
        Ok(())
    }

    fn nuevo_id_conexion(&mut self) -> IdConexion {
        self.ultimo_id_conexion += 1;
        self.ultimo_id_conexion
    }

    pub fn iniciar(mut servidor: Servidor) -> JoinHandle<()> {
        thread::spawn(move || {
            servidor.inicio();
        })
    }

    pub fn inicio(&mut self) {
        let direccion: String = self
            .configuracion
            .obtener::<String>("direccion")
            .unwrap_or("127.0.0.1".to_string());

        let puerto: u16 = self.configuracion.obtener::<u16>("puerto").unwrap_or(4222);

        let listener: TcpListener = TcpListener::bind(format!("{}:{}", direccion, puerto)).unwrap();
        listener
            .set_nonblocking(true) // Hace que el listener no bloquee el hilo principal
            .expect("No se pudo poner el listener en modo no bloqueante");

        loop {
            match listener.accept() {
                // Si escucho algo, genero una nueva conexion
                Ok((stream, _)) => {
                    stream.set_nonblocking(true).unwrap();

                    // Creamos una copia del logger para la nueva conexion
                    let mut registrador_para_nueva_conexion: Registrador = self.registrador.clone();
                    // Establecemos el hilo actual para la nueva conexion
                    registrador_para_nueva_conexion.establecer_hilo(self.proximo_id_hilo as IdHilo);

                    // Generamos un nuevo id único para la nueva conexión
                    let id_conexion: u64 = self.nuevo_id_conexion();

                    let conexion: Conexion = Conexion::new(
                        id_conexion,
                        Box::new(stream),
                        registrador_para_nueva_conexion,
                        self.cuentas.clone(),
                    );

                    let (tx, _) = &self.hilos[self.proximo_id_hilo];
                    match tx.send((id_conexion, conexion)) {
                        // Envio la conexion al hilo
                        Ok(_) => {
                            self.proximo_id_hilo = (self.proximo_id_hilo + 1) % self.hilos.len();
                        }
                        Err(e) => {
                            panic!("Error: {}", e);
                        }
                    }

                    thread::sleep(Duration::from_micros(500));
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // No hay conexiones nuevas
                }
                Err(e) => {
                    panic!("Error: {}", e);
                }
            }
        }
    }
}
