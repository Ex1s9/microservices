use tonic::transport::Server;
use dotenv::dotenv;

pub mod game {
    tonic::include_proto!("game");
}

mod types;
mod grpc_service;
mod handlers;
mod routes;

use crate::grpc_service::GameServiceImpl;
use crate::routes::create_routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let grpc_addr = "[::1]:50052".parse()?;
    let http_addr = "0.0.0.0:8080".parse::<std::net::SocketAddr>()?;
    
    let game_service = GameServiceImpl;

    let app = create_routes();

    let http_server = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(&http_addr).await.unwrap();
        println!("HTTP API server listening on http://{}", http_addr);
        axum::serve(listener, app).await.unwrap();
    });

    let grpc_server = tokio::spawn(async move {
        println!("gRPC service listening on {}", grpc_addr);
        Server::builder()
            .add_service(game::game_service_server::GameServiceServer::new(
                game_service,
            ))
            .serve(grpc_addr)
            .await
            .unwrap();
    });

    tokio::select! {
        _ = http_server => println!("HTTP server finished"),
        _ = grpc_server => println!("gRPC server finished"),
    }

    Ok(())
}