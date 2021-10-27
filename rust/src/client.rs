use async_stream::stream;
use futures::stream::TryStreamExt;
use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;
use tokio::time;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

async fn say_hello(
    mut client: GreeterClient<tonic::transport::Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let response = client.say_hello(request).await?;
    let msg = response.into_inner();
    log::info!("{:?}", msg);
    Ok(())
}

async fn server_streaming(
    mut client: GreeterClient<tonic::transport::Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let response = client.server_streaming(request).await?;
    let mut stream = response.into_inner();
    while let Some(msg) = stream.try_next().await? {
        log::info!("{:?}", msg);
    }
    Ok(())
}

async fn client_streaming(
    mut client: GreeterClient<tonic::transport::Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = stream! {
        for value in ["1", "2", "3", "4", "5"] {
            tokio::time::sleep(time::Duration::from_secs(1)).await;

            yield HelloRequest {
                name: value.into(),
            };
        }
    };
    let response = client.client_streaming(request).await?;
    let msg = response.into_inner();
    log::info!("{:?}", msg);
    Ok(())
}

async fn bidirectional_streaming(
    mut client: GreeterClient<tonic::transport::Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = stream! {
        for value in ["1", "2", "3", "4", "5"] {
            tokio::time::sleep(time::Duration::from_secs(1)).await;

            yield HelloRequest {
                name: value.into(),
            };
        }
    };
    let response = client.bidirectional_streaming(request).await?;
    let mut stream = response.into_inner();
    while let Some(msg) = stream.try_next().await? {
        log::info!("{:?}", msg);
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();
    println!("{:?}", args);

    let client = GreeterClient::connect("http://[::1]:50051").await?;

    match &*args[1] {
        "1" => say_hello(client).await,
        "2" => server_streaming(client).await,
        "3" => client_streaming(client).await,
        "4" => bidirectional_streaming(client).await,
        _ => Ok(()),
    }
}
