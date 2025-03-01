use redis::{AsyncCommands, Client};
use std::time::Duration;
use tokio::fs;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let path = std::env::var("REDIS_HOST")?;

    let now = std::time::Instant::now();
    // Wait until the file exists
    while fs::metadata(&path).await.is_err() && now.elapsed().as_secs() < 30 {
        println!("Waiting for the file to exist: {}", path);
        sleep(Duration::from_secs(1)).await;
    }

    let params = format!("redis+unix://{path}");
    println!("Connecting to Redis: {}", params);
    let client = Client::open(params)?;

    let mut conn = client.get_multiplexed_async_connection().await?;

    conn.set_ex::<_, _, ()>(3, 5, 10).await?;

    Ok(())
}
