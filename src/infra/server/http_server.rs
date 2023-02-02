use crate::infra::server::shutdown_control::ShutDownControl;
use crate::infra::server::{HttpFilter, HttpHandle, ShutDown};
use hyper::server::conn::{AddrIncoming, AddrStream};
use hyper::server::Builder;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use wd_run::Context;

pub struct HyperHttpServer {
    filters: Vec<Box<dyn HttpFilter + Send + Sync + 'static>>,
    handle: Box<dyn HttpHandle + Send + Sync + 'static>,
}

impl Default for HyperHttpServer {
    fn default() -> Self {
        HyperHttpServer::new(|_c, _r| async { Ok(Response::new(Body::from(r#"{"health":true}"#))) })
    }
}
impl From<Box<dyn HttpHandle + Send + Sync + 'static>> for HyperHttpServer {
    fn from(value: Box<dyn HttpHandle + Send + Sync + 'static>) -> Self {
        let mut server = HyperHttpServer::default();
        server.handle = value;
        server
    }
}
impl HyperHttpServer {
    pub fn new<H: HttpHandle + Send + Sync + 'static>(handle: H) -> Self {
        let filters = vec![];
        let handle = Box::new(handle);
        Self { filters, handle }
    }
    pub fn append_handles(
        mut self,
        mut filters: Vec<Box<dyn HttpFilter + Send + Sync + 'static>>,
    ) -> Self {
        self.filters.append(&mut filters);
        self
    }
    pub async fn service(
        self: Arc<HyperHttpServer>,
        req: Request<Body>,
    ) -> Result<Response<Body>, Infallible> {
        let ctx = Context::new();
        let mut index = 0;
        let mut req = Some(req);
        let mut resp = None; //Response::new(Body::empty());

        for (i, filter) in self.filters.iter().enumerate() {
            index = i;
            req = match filter.request(ctx.clone(), req.unwrap()).await {
                Ok(o) => Some(o),
                Err(e) => {
                    resp = Some(e);
                    None
                }
            };
            if req.is_none() {
                break;
            }
        }
        if req.is_some() {
            let result = self.handle.handle(ctx.clone(), req.unwrap()).await;
            resp = match result {
                Ok(o) => Some(o),
                Err(e) => {
                    wd_log::log_error_ln!("HyperHttpServer service handle error:{}", e);
                    Some(
                        Response::builder()
                            .status(500)
                            .body(Body::from("unknown error"))
                            .unwrap(),
                    )
                }
            };
        }
        let skip = self.filters.len() - index - 1;
        for (_, filter) in self
            .filters
            .iter()
            .rev()
            .enumerate()
            .filter(|(i, _)| i >= &skip)
        {
            resp = Some(filter.response(ctx.clone(), resp.unwrap()).await)
        }
        Ok(resp.unwrap())
    }
}

pub struct HyperHttpServerBuilder {
    addr: SocketAddr,
    filters: Vec<Box<dyn HttpFilter + Send + Sync + 'static>>,
    handle: Option<Box<dyn HttpHandle + Send + Sync + 'static>>,
    shutdown: Option<ShutDownControl>,
}

impl HyperHttpServerBuilder {
    pub fn new() -> Self {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6789);
        let filters = vec![];
        let handle = None;
        let shutdown = None;
        Self {
            addr,
            filters,
            handle,
            shutdown,
        }
    }
    #[allow(dead_code)]
    pub fn set_addr(mut self, addr: SocketAddr) -> Self {
        self.addr = addr;
        self
    }
    pub fn shutdown_singe(mut self) -> (Self, ShutDown) {
        if self.shutdown.is_some() {
            let sd = self.shutdown.as_ref().unwrap().generate_shutdown();
            return (self, sd);
        }
        let sdc = ShutDownControl::new();
        let sd = sdc.generate_shutdown();
        self.shutdown = Some(sdc);
        (self, sd)
    }
    pub fn set_shutdown_singe(mut self, sd: ShutDown) -> Self {
        self.shutdown = Some(ShutDownControl::from(sd));
        self
    }
    pub fn handle<H: HttpHandle + Send + Sync + 'static>(mut self, handle: H) -> Self {
        self.handle = Some(Box::new(handle));
        self
    }
    #[allow(dead_code)]
    pub fn parse_addr<S: ToSocketAddrs<Iter = SocketAddr>>(
        mut self,
        addr: S,
    ) -> anyhow::Result<Self> {
        self.addr = addr.to_socket_addrs()?;
        Ok(self)
    }
    pub fn append_filter<H: HttpFilter + Send + Sync + 'static>(mut self, filter: H) -> Self {
        self.filters.push(Box::new(filter));
        self
    }
    pub async fn custom_run<
        C: FnOnce(Builder<AddrIncoming>) -> anyhow::Result<Builder<AddrIncoming>> + Send,
    >(
        self,
        custom_func: C,
    ) -> anyhow::Result<()> {
        if self.handle.is_none() {
            return Err(anyhow::anyhow!(
                "HyperHttpServerBuilder handle function is nil,please impl HttpHandle"
            ));
        }
        let ser =
            Arc::new(HyperHttpServer::from(self.handle.unwrap()).append_handles(self.filters));
        let http_service = make_service_fn(move |_c: &AddrStream| {
            let ser = ser.clone();
            let service = service_fn(move |req| HyperHttpServer::service(ser.clone(), req));
            async move { Ok::<_, Infallible>(service) }
        });
        let server = Server::bind(&self.addr);
        let server = custom_func(server)?.serve(http_service);
        if let Some(sdc) = self.shutdown {
            let asdc = sdc.clone();
            let server = server.with_graceful_shutdown(async move {
                asdc.wait().await;
            });
            let result = server.await;
            sdc.down().await;
            result?;
            return Ok(());
        }
        server.await?;
        Ok(())
    }
    #[allow(dead_code)]
    pub async fn run(self) -> anyhow::Result<()> {
        self.custom_run(|b| Ok(b)).await
    }
    #[allow(dead_code)]
    pub fn async_run(self) -> ShutDown {
        let (server, sd) = self.shutdown_singe();
        tokio::spawn(async move {
            server
                .custom_run(|b| Ok(b))
                .await
                .expect("async_run http server error")
        });
        return sd;
    }
}
