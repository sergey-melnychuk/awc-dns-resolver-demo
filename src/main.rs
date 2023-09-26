// https://github.com/actix/actix-net/blob/master/actix-tls/src/connect/resolve.rs

use std::net::SocketAddr;

use actix_tls::connect::{Resolve, Resolver};
use futures_util::future::LocalBoxFuture;

// use trust-dns async tokio resolver
use trust_dns_resolver::TokioAsyncResolver;

struct MyResolver {
    trust_dns: TokioAsyncResolver,
}

// impl Resolve trait and convert given host address str and port to SocketAddr.
impl Resolve for MyResolver {
    fn lookup<'a>(
        &'a self,
        host: &'a str,
        port: u16,
    ) -> LocalBoxFuture<'a, Result<Vec<SocketAddr>, Box<dyn std::error::Error>>> {
        Box::pin(async move {
            let res = self
                .trust_dns
                .lookup_ip(host)
                .await?
                .iter()
                .map(|ip| SocketAddr::new(ip, port))
                .collect();
            println!("host={host} port={port}: {:?}", res);
            Ok(res)
        })
    }
}

#[actix_web::main]
async fn main() {
    let trust_dns = TokioAsyncResolver::tokio_from_system_conf().unwrap();
    let my_resolver = MyResolver { trust_dns };
    
    // wrap custom resolver
    let resolver = Resolver::custom(my_resolver);
        
    // resolver can be passed to connector factory where returned service factory
    // can be used to construct new connector services for use in clients
    let factory = actix_tls::connect::Connector::new(resolver);
    let connector = factory.service();

    // https://github.com/actix/actix-web/blob/master/awc/src/client/connector.rs#L944
    let connector = awc::Connector::new().connector(connector);

    let client = awc::Client::builder().connector(connector).finish();

    let request = client.get("http://www.example.com").send();
    let mut response = request.await.unwrap();

    println!("response.status: {}", response.status());
    println!("response: {:?}", response.body().await.unwrap());
}
