#[allow(unused_imports)]
use hyper::server::conn::AddrIncoming;
#[allow(unused_imports)]
use hyper::server::Server as HyperServer;
use listenfd::ListenFd;
use std::net::{IpAddr, SocketAddr, TcpListener};
use std::sync::Arc;
use tokio::sync::oneshot::Receiver;

use crate::handler::{RequestHandler, RequestHandlerOpts};
#[cfg(any(unix, windows))]
use crate::signals;
#[cfg(feature = "tls")]
use crate::tls::{TlsAcceptor, TlsConfigBuilder};
use crate::{cors, helpers, logger, Settings};
use crate::{service::RouterService, Context, Result};


pub struct Server {
    opts: Settings,
    worker_threads: usize,
    max_blocking_threads: usize,
}

impl Server {
   
    pub fn new() -> Result<Server> {
        let opts = Settings::get()?;

        let cpus = num_cpus::get();
        let worker_threads = match opts.general.threads_multiplier {
            0 | 1 => cpus,
            n => cpus * n,
        };
        let max_blocking_threads = opts.general.max_blocking_threads;

        Ok(Server {
            opts,
            worker_threads,
            max_blocking_threads,
        })
    }

    /// Run the multi-thread `Server` as standalone.
    /// It is a top-level function of [run_server_on_rt](#method.run_server_on_rt).
    pub fn run_standalone(self) -> Result {
        // Logging system initialization
        logger::init(&self.opts.general.log_level)?;

        self.run_server_on_rt(None, || {})
    }

    /// Run the multi-thread `Server` which will be used by a Windows service.
    /// It is a top-level function of [run_server_on_rt](#method.run_server_on_rt).
    #[cfg(windows)]
    pub fn run_as_service<F>(self, cancel: Option<Receiver<()>>, cancel_fn: F) -> Result
    where
        F: FnOnce(),
    {
        self.run_server_on_rt(cancel, cancel_fn)
    }

    pub fn run_server_on_rt<F>(self, cancel_recv: Option<Receiver<()>>, cancel_fn: F) -> Result
    where
        F: FnOnce(),
    {
        tracing::debug!(%self.worker_threads, "initializing tokio runtime with multi thread scheduler");

        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(self.worker_threads)
            .max_blocking_threads(self.max_blocking_threads)
            .thread_name("static-web-server")
            .enable_all()
            .build()?
            .block_on(async {
                tracing::trace!("starting web server");
                if let Err(err) = self.start_server(cancel_recv, cancel_fn).await {
                    tracing::error!("server failed to start up: {:?}", err);
                    std::process::exit(1)
                }
            });

        tracing::trace!("runtime initialized");

        Ok(())
    }

    
    async fn start_server<F>(self, _cancel_recv: Option<Receiver<()>>, _cancel_fn: F) -> Result
    where
        F: FnOnce(),
    {
        // Config "general" options
        let general = self.opts.general;

        // Config-file "advanced" options
        let advanced_opts = self.opts.advanced;

        // Config file option
        if let Some(config_file) = general.config_file {
            let config_file = helpers::adjust_canonicalization(&config_file);
            tracing::info!("config file: {}", config_file);
        }

      
        let (tcp_listener, addr_str);
        match general.fd {
            Some(fd) => {
                addr_str = format!("@FD({fd})");
                tcp_listener = ListenFd::from_env()
                    .take_tcp_listener(fd)?
                    .with_context(|| "failed to convert inherited 'fd' into a 'tcp' listener")?;
                tracing::info!(
                    "converted inherited file descriptor {} to a 'tcp' listener",
                    fd
                );
            }
            None => {
                let ip = general
                    .host
                    .parse::<IpAddr>()
                    .with_context(|| format!("failed to parse {} address", general.host))?;
                let addr = SocketAddr::from((ip, general.port));
                tcp_listener = TcpListener::bind(addr)
                    .with_context(|| format!("failed to bind to {addr} address"))?;
                addr_str = addr.to_string();
                tracing::info!("server bound to tcp socket {}", addr_str);
            }
        }

       
        let root_dir = helpers::get_valid_dirpath(&general.root)
            .with_context(|| "root directory was not found or inaccessible")?;

    
        let page404 = helpers::read_bytes_default(&general.page404);
        let page50x = helpers::read_bytes_default(&general.page50x);

      
        let page_fallback = helpers::read_bytes_default(&general.page_fallback.unwrap_or_default());

       
        let threads = self.worker_threads;
        tracing::info!("runtime worker threads: {}", threads);

        
        tracing::info!(
            "runtime max blocking threads: {}",
            general.max_blocking_threads
        );

        // Security Headers option
        let security_headers = general.security_headers;
        tracing::info!("security headers: enabled={}", security_headers);

        // Auto compression based on the `Accept-Encoding` header
        let compression = general.compression;
        tracing::info!("auto compression: enabled={}", compression);

        // Check pre-compressed files based on the `Accept-Encoding` header
        let compression_static = general.compression_static;
        tracing::info!("compression static: enabled={}", compression_static);

        // Directory listing option
        let dir_listing = general.directory_listing;
        tracing::info!("directory listing: enabled={}", dir_listing);

        // Directory listing order number
        let dir_listing_order = general.directory_listing_order;
        tracing::info!("directory listing order code: {}", dir_listing_order);

        // Directory listing format
        let dir_listing_format = general.directory_listing_format;
        tracing::info!("directory listing format: {}", dir_listing_format);

        // Cache control headers option
        let cache_control_headers = general.cache_control_headers;
        tracing::info!("cache control headers: enabled={}", cache_control_headers);

        
        let cors = cors::new(
            general.cors_allow_origins.trim(),
            general.cors_allow_headers.trim(),
            general.cors_expose_headers.trim(),
        );

  
        let basic_auth = general.basic_auth.trim().to_owned();
        tracing::info!(
            "basic authentication: enabled={}",
            !general.basic_auth.is_empty()
        );

        
        let log_remote_address = general.log_remote_address;
        tracing::info!("log remote address: enabled={}", log_remote_address);

        // Log redirect trailing slash option
        let redirect_trailing_slash = general.redirect_trailing_slash;
        tracing::info!(
            "redirect trailing slash: enabled={}",
            redirect_trailing_slash
        );

        // Ignore hidden files option
        let ignore_hidden_files = general.ignore_hidden_files;
        tracing::info!("ignore hidden files: enabled={}", ignore_hidden_files);

        // Grace period option
        let grace_period = general.grace_period;
        tracing::info!("grace period before graceful shutdown: {}s", grace_period);

        // Create a service router for Hyper
        let router_service = RouterService::new(RequestHandler {
            opts: Arc::from(RequestHandlerOpts {
                root_dir,
                compression,
                compression_static,
                dir_listing,
                dir_listing_order,
                dir_listing_format,
                cors,
                security_headers,
                cache_control_headers,
                page404,
                page50x,
                page_fallback,
                basic_auth,
                log_remote_address,
                redirect_trailing_slash,
                ignore_hidden_files,
                advanced_opts,
            }),
        });

  
      
}
