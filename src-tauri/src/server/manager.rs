use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{mpsc, Mutex, OnceLock},
    thread::{self, JoinHandle},
};

use tokio::sync::oneshot;

struct RunningServer {
    shutdown: oneshot::Sender<()>,
    thread: JoinHandle<()>,
}

fn servers() -> &'static Mutex<HashMap<String, RunningServer>> {
    static SERVERS: OnceLock<Mutex<HashMap<String, RunningServer>>> = OnceLock::new();
    SERVERS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Starts an Axum server on its own OS thread.
///
/// The function returns only after the listener has been bound, so a successful
/// result means the server can accept connections. `end` uses `name` to signal
/// a graceful shutdown and wait for this thread to exit.
pub fn start(name: &str, app: axum::Router, port: Option<u16>) -> Result<SocketAddr, String> {
    if name.is_empty() {
        return Err("server name must not be empty".into());
    }

    let port = port.unwrap_or(0);

    // Keep the lock through startup so two callers cannot register the same
    // name while the first server is still binding its listener.
    let mut servers = servers()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if servers.contains_key(name) {
        return Err(format!("server '{name}' is already running"));
    }

    let (shutdown, shutdown_signal) = oneshot::channel();
    let (ready, ready_signal) = mpsc::sync_channel::<Result<SocketAddr, String>>(1);
    let thread_name = name.to_owned();
    let address = format!("0.0.0.0:{port}");
    let server_thread = thread::Builder::new()
        .name(format!("axum-{name}"))
        .spawn(move || {
            let runtime = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    let _ = ready.send(Err(format!("failed to create server runtime: {error}")));
                    return;
                }
            };

            runtime.block_on(async move {
                let listener = match tokio::net::TcpListener::bind(&address).await {
                    Ok(listener) => listener,
                    Err(error) => {
                        let _ = ready.send(Err(format!("failed to bind {address}: {error}")));
                        return;
                    }
                };

                if ready.send(Ok(listener.local_addr().unwrap())).is_err() {
                    return;
                }

                tracing::info!(name = %thread_name, address = %address, "Axum server started");
                if let Err(error) = axum::serve(listener, app)
                    .with_graceful_shutdown(async {
                        let _ = shutdown_signal.await;
                    })
                    .await
                {
                    tracing::error!(name = %thread_name, error = %error, "Axum server stopped with an error");
                }
                tracing::info!(name = %thread_name, "Axum server stopped");
            });
        })
        .map_err(|error| format!("failed to spawn server thread: {error}"))?;

    match ready_signal.recv() {
        Ok(Ok(adrr)) => {
            servers.insert(
                name.to_owned(),
                RunningServer {
                    shutdown,
                    thread: server_thread,
                },
            );
            Ok(adrr)
        }
        Ok(Err(error)) => {
            let _ = server_thread.join();
            Err(error)
        }
        Err(_) => {
            let _ = server_thread.join();
            Err("server thread exited before startup completed".into())
        }
    }
}

/// Gracefully stops a server previously started with [`start`].
pub fn end(name: &str) -> Result<(), String> {
    let server = {
        let mut servers = servers()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        servers
            .remove(name)
            .ok_or_else(|| format!("server '{name}' is not running"))?
    };

    // A send error only means the server already exited; joining still cleans
    // up the OS thread and makes the port available again.
    let _ = server.shutdown.send(());
    server
        .thread
        .join()
        .map_err(|_| format!("server '{name}' thread panicked"))
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::TcpListener,
    };

    use axum::{routing::get, Router};

    use super::{end, start};

    fn available_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind temporary listener");
        listener.local_addr().expect("read local address").port()
    }

    #[test]
    fn starts_on_a_background_thread_and_stops() {
        let port = available_port();
        let name = "manager-starts-and-stops";
        let app = Router::new().route("/", get(|| async { "ok" }));

        start(name, app, Some(port)).expect("start server");

        let mut stream = std::net::TcpStream::connect(("127.0.0.1", port)).expect("connect");
        stream
            .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .expect("send request");
        let mut response = String::new();
        stream.read_to_string(&mut response).expect("read response");
        assert!(response.starts_with("HTTP/1.1 200"), "{response}");

        end(name).expect("stop server");
        assert!(std::net::TcpStream::connect(("127.0.0.1", port)).is_err());
    }

    #[test]
    fn rejects_a_duplicate_server_name() {
        let name = "manager-rejects-duplicates";
        start(name, Router::new(), Some(available_port())).expect("start server");

        let error = start(name, Router::new(), Some(available_port()))
            .expect_err("duplicate name must fail");
        assert!(error.contains("already running"));

        end(name).expect("stop server");
    }
}
