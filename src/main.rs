use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() {


    
    let url = "https://hasura-mainnet.nfteller.org/v1/graphql"; // Replace with your actual endpoint
    let client = reqwest::Client::new();
    
    println!("Starting periodic POST requests every hour...");
    
    let mut interval = time::interval(Duration::from_secs(3600)); // 1 hour = 3600 seconds
    
    loop {
        interval.tick().await;
        
        match make_post_request(&client, url).await {
            Ok(response) => {
                println!("POST request successful:");
                println!("{}", response);
            }
            Err(e) => {
                eprintln!("POST request failed: {}", e);
            }
        }
    }
}

async fn make_post_request(client: &reqwest::Client, url: &str) -> Result<String, reqwest::Error> {
    let body = serde_json::json!({
        "query": "query MyQuery { cursors { block_id block_num cursor id } }"
    });
    
    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;
    
    let text = response.text().await?;
    Ok(text)
}
