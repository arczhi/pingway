mod pkg;

use std::env;
use std::io::Read;
use bytes::Bytes;
use http::Method;
use pkg::{config,validate,log};
use std::process::{self, Termination};
use std::path::Path;
use async_trait::async_trait;
use pingora::prelude::*;
use pingora_proxy::{http_proxy_service, ProxyHttp, Session};
use std::collections::HashMap;
use std::sync::{Arc,Mutex};
use prometheus;
use chrono::{DateTime, Utc};


pub struct Pingway{
    access_log: Arc<Mutex<log::Log>>,
    upstream: HashMap<String,String>,
    prometheus_enabled: bool,
    req_counter: prometheus::IntCounter,
}
pub struct PingwayCtx {
    req_start_time: DateTime<Utc>,
    req_end_time: DateTime<Utc>,
    req_uri: String,
    request_body: Bytes,
    response_body: Bytes,
}

#[async_trait]
impl ProxyHttp for Pingway {
    type CTX = PingwayCtx;
    fn new_ctx(&self) -> Self::CTX{
        PingwayCtx {
            req_start_time: Utc::now(),
            req_end_time: Utc::now(),
            req_uri: String::new(),
            request_body: Bytes::new(),
            response_body: Bytes::new(),
        }
    }

    // request control
    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool>{
        ctx.req_start_time = Utc::now();
        session.req_header_mut().insert_header("GATEWAY-TAG", "PINGWAY").unwrap();
        Ok(false)
    }

    // upstream proxy
    async fn upstream_peer(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let clone_req_header = session.req_header().clone(); // avoid double borrow with variable session
        let uri = clone_req_header.uri.path();
        let mut backend_addr = "";
        let uri_parts:Vec<&str> = uri.split("/").collect();
        let uri_prefix = &format!("/{}",uri_parts[1]);
        
        if let Some(addr) = self.upstream.get(uri_prefix) {
            if uri.starts_with(uri_prefix) {
                let new_uri:http::Uri;
                if let Err(err) = uri.replace(uri_prefix, "").parse::<http::Uri>() {
                    // uri like /cloud
                    new_uri = "/".parse::<http::Uri>().unwrap();
                }else {
                    // uri like /cloud/123
                    new_uri = uri.replace(uri_prefix, "").parse::<http::Uri>().unwrap();
                }
                // println!("uri: {uri} uri_prefix: {uri_prefix} new_uri: {new_uri}");
                session.req_header_mut().set_uri(new_uri);
                backend_addr = addr;
            }
        }else{
            backend_addr = "0.0.0.0:6199";
        }
        let peer = Box::new(HttpPeer::new(backend_addr, config::USE_SSL, "one.one.one.one".to_string()));
        Ok(peer)
    }

    // request body
    async fn upstream_request_filter(
        &self,
        session: &mut Session,
        upstream_request: &mut RequestHeader,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        if let Ok(body) = session.read_request_body().await {
            if let Some(body_bytes) = body {
                ctx.request_body = body_bytes;
            }
        };
        ctx.req_uri = String::from_utf8_lossy(upstream_request.raw_path()).into_owned();
        Ok(())
    }

    // response body
    fn upstream_response_body_filter(
        &self,
        session: &mut Session,
        body: &mut Option<Bytes>,
        end_of_stream: bool,
        ctx: &mut Self::CTX,
    ) {
        ctx.response_body = <std::option::Option<bytes::Bytes> as Clone>::clone(&body).unwrap();
    }

    // access log
    async fn logging(&self, session: &mut Session, _e: Option<&Error>, ctx: &mut Self::CTX){
        let req_summary = session.request_summary();
        let req_header = session.req_header().headers.clone();
        let req_time = ctx.req_start_time.format("%Y-%m-%d %H:%M:%S%.3f");
        ctx.req_end_time = Utc::now();
        let req_cost_time = (ctx.req_end_time.timestamp_millis() - ctx.req_start_time.timestamp_millis()).to_string();
        let req_uri = &ctx.req_uri;
        let req_body_byte  = ctx.request_body.to_vec();
        let req_body = String::from_utf8_lossy(&req_body_byte);
        let resp_body_byte = ctx.response_body.to_vec();
        let resp_body = String::from_utf8_lossy(&resp_body_byte);
        let log_content = format!("[{req_time}] {req_summary} , cost: {req_cost_time} ms , req_uri: {req_uri} , header: {req_header:?} , req_body: {req_body:?}, resp: {resp_body:?} \n");
        let _ = self.access_log.lock().unwrap().write_all(log_content.as_bytes()); // write_data
        if self.prometheus_enabled {
            self.req_counter.inc();
        }
    }

}

fn main() {

    let conf_path =  Opt::default().conf.unwrap();
    let config = config::load_config_from_file(&conf_path).unwrap();
    let current_dir = env::current_dir().expect("current dir error");
    let conf_path = if let Some(p) = conf_path.strip_prefix("./") {p} else { &"" }; // "".as_ref(); // 将 "" 转换为 &'static str 类型的引用
    let abs_path = Path::new(&current_dir).join(conf_path);
    let abs_config_path = abs_path.to_str().unwrap();

    let mut server = Server::new(Some(Opt::default())).unwrap();
    server.bootstrap();

    let mut upstream_map = HashMap::new();
    for v in config.upstream.iter() {
        // check addr
        let addr = &v.addr;
        if let Err(err) = validate::validate_ip_and_port(addr){
            println!("[Pingway] start error: {err} , check addr: {addr}");
            process::exit(1);
        };
        upstream_map.insert(v.uri_prefix.to_string(), v.addr.to_string());
    }

    let pingway = Pingway{
        access_log: Arc::new(Mutex::new(log::Log::new(&config.access_log))),
        upstream: upstream_map,
        prometheus_enabled: config.prometheus.enabled,
        req_counter: prometheus::register_int_counter!("gateway_req_total", "total number of requests").unwrap()
    };

    let mut pingway_service = http_proxy_service(&server.configuration, pingway);
    let server_addr = &format!("0.0.0.0:{}",config.port);
    pingway_service.add_tcp(server_addr);
    server.add_service(pingway_service);

    //prometheus metrics
    if config.prometheus.enabled {
        let mut prometheus_service_http =
        pingora::services::listening::Service::prometheus_http_service();
        prometheus_service_http.add_tcp(&format!("0.0.0.0:{}",config.prometheus.port));
        server.add_service(prometheus_service_http);
    }
     

    let pid = process::id();
    println!("[Pingway] a gateway scratch based on pingora !");
    println!("[Pingway] starting ...");
    println!("[Pingway] config file path: {abs_config_path}" );
    println!("[Pingway] listening on: {server_addr}");
    println!("[Pingway] enter 'CTRL+C' to shutdown or enter 'kill {pid}' to gracefully shutdown");

    server.run_forever();
}
