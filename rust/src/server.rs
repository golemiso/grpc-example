pub mod hello_world {
    tonic::include_proto!("helloworld");
}

use async_stream::try_stream;
use futures::stream::{self, Stream, StreamExt, TryStreamExt};
use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};
use std::pin::Pin;
use tokio::time;
use tonic::{transport::Server, Request, Response, Status, Streaming};

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request: {:?}", request);

        let request = request.into_inner();
        let reply = HelloReply {
            message: format!("Hello {}!", request.name),
        };

        Ok(Response::new(reply))
    }

    type ServerStreamingStream =
        Pin<Box<dyn Stream<Item = Result<HelloReply, Status>> + Send + Sync>>;

    async fn server_streaming(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<Self::ServerStreamingStream>, Status> {
        println!("Got a request: {:?}", request);

        let request = request.into_inner();
        let stream = try_stream! {
            for _ in 0..3 {
                tokio::time::sleep(time::Duration::from_secs(1)).await;

                yield HelloReply {
                    message: format!("Hello {}!", request.name).into(),
                };
            }
        };

        Ok(Response::new(
            Box::pin(stream) as Self::ServerStreamingStream
        ))
    }

    async fn client_streaming(
        &self,
        request: Request<Streaming<HelloRequest>>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request: {:?}", request);

        let mut stream = request.into_inner();
        let mut i = 0;
        while stream.try_next().await?.is_some() {
            i += 1;
        }
        let reply = HelloReply {
            message: format!("Hello {}!", i),
        };

        Ok(Response::new(reply))
    }

    type BidirectionalStreamingStream =
        Pin<Box<dyn Stream<Item = Result<HelloReply, Status>> + Send + Sync>>;

    async fn bidirectional_streaming(
        &self,
        request: Request<Streaming<HelloRequest>>,
    ) -> Result<Response<Self::BidirectionalStreamingStream>, Status> {
        println!("Got a request: {:?}", request);

        let mut stream = request.into_inner();
        if let Some(first_msg) = stream.message().await? {
            let single_message = stream::iter(vec![Ok(first_msg)]);
            let mut stream = single_message.chain(stream);

            let stream = try_stream! {
                while let Some(msg) = stream.try_next().await? {

                    //tokio::time::sleep(time::Duration::from_micros(param.interval_us as u64)).await;

                    yield HelloReply {
                        message: format!("Hello {}!", msg.name).into(),
                    };
                }
            };

            Ok(Response::new(
                Box::pin(stream) as Self::BidirectionalStreamingStream
            ))
        } else {
            let stream = stream::empty();
            Ok(Response::new(
                Box::pin(stream) as Self::BidirectionalStreamingStream
            ))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::]:50051".parse()?;
    let greeter = MyGreeter::default();

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
