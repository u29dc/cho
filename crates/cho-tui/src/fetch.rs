//! Background fetch worker for non-blocking route loading.

use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::Instant;

use crate::api::{ApiEngine, FetchContext, RouteLoadOptions, RoutePayload};
use crate::cache::CacheKey;
use crate::routes::RouteDefinition;

/// Trigger source for a route load.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadReason {
    /// Initial route load at startup.
    Startup,
    /// Left-nav settled background revalidation.
    NavRevalidate,
    /// Left-nav Enter explicit reload.
    NavEnterReload,
    /// Palette "navigate" action.
    PaletteNavigate,
    /// Manual refresh key or command.
    ManualRefresh,
    /// Context changed via prompt/filter.
    ContextChanged,
}

/// Background request payload.
#[derive(Debug, Clone)]
pub struct FetchRequest {
    /// Monotonic request id assigned by app.
    pub request_id: u64,
    /// Target route definition.
    pub route: RouteDefinition,
    /// Target context snapshot.
    pub context: FetchContext,
    /// Load options.
    pub options: RouteLoadOptions,
    /// Route cache key for stale-response suppression.
    pub cache_key: CacheKey,
    /// Trigger source.
    pub reason: LoadReason,
}

/// Background result payload.
#[derive(Debug)]
pub struct FetchResponse {
    /// Monotonic request id assigned by app.
    pub request_id: u64,
    /// Route id that was fetched.
    pub route_id: String,
    /// Route cache key tied to this result.
    pub cache_key: CacheKey,
    /// Trigger source.
    pub reason: LoadReason,
    /// Fetch elapsed milliseconds.
    pub elapsed_ms: u64,
    /// Route payload or error string.
    pub payload: Result<RoutePayload, String>,
}

#[derive(Debug)]
enum WorkerCommand {
    Fetch(Box<FetchRequest>),
    Shutdown,
}

/// Single-threaded fetch worker with request/result channels.
pub struct FetchWorker {
    command_tx: Sender<WorkerCommand>,
    result_rx: Receiver<FetchResponse>,
    join_handle: Option<JoinHandle<()>>,
}

impl FetchWorker {
    /// Starts a worker thread.
    pub fn new() -> Result<Self, String> {
        let (command_tx, command_rx) = mpsc::channel::<WorkerCommand>();
        let (result_tx, result_rx) = mpsc::channel::<FetchResponse>();

        let join_handle = thread::Builder::new()
            .name("cho-tui-fetch-worker".to_string())
            .spawn(move || run_worker(command_rx, result_tx))
            .map_err(|e| format!("failed to start fetch worker: {e}"))?;

        Ok(Self {
            command_tx,
            result_rx,
            join_handle: Some(join_handle),
        })
    }

    /// Sends a fetch request.
    pub fn request(&self, request: FetchRequest) -> Result<(), String> {
        self.command_tx
            .send(WorkerCommand::Fetch(Box::new(request)))
            .map_err(|e| format!("failed to queue fetch request: {e}"))
    }

    /// Polls one result if available.
    pub fn try_recv(&self) -> Option<FetchResponse> {
        match self.result_rx.try_recv() {
            Ok(result) => Some(result),
            Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => None,
        }
    }
}

impl Drop for FetchWorker {
    fn drop(&mut self) {
        let _ = self.command_tx.send(WorkerCommand::Shutdown);
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }
}

fn run_worker(command_rx: Receiver<WorkerCommand>, result_tx: Sender<FetchResponse>) {
    let mut api = None::<ApiEngine>;
    let mut init_error = None::<String>;

    while let Ok(command) = command_rx.recv() {
        match command {
            WorkerCommand::Shutdown => break,
            WorkerCommand::Fetch(request) => {
                if api.is_none() && init_error.is_none() {
                    match ApiEngine::new() {
                        Ok(engine) => api = Some(engine),
                        Err(err) => init_error = Some(err),
                    }
                }

                let started = Instant::now();
                let payload = if let Some(engine) = api.as_ref() {
                    engine.fetch_route_with_options(
                        &request.route,
                        &request.context,
                        request.options,
                    )
                } else {
                    Err(format!(
                        "fetch worker initialization failed: {}",
                        init_error
                            .clone()
                            .unwrap_or_else(|| "unknown error".to_string())
                    ))
                };

                let response = FetchResponse {
                    request_id: request.request_id,
                    route_id: request.route.id.clone(),
                    cache_key: request.cache_key,
                    reason: request.reason,
                    elapsed_ms: started.elapsed().as_millis() as u64,
                    payload,
                };
                if result_tx.send(response).is_err() {
                    break;
                }
            }
        }
    }
}
