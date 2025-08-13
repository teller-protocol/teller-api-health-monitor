use std::time::Duration;
use std::env;
use tokio::time;
use ethers_rs::types::U256;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file if it exists
    dotenvy::dotenv().ok();
    
    println!("Starting periodic POST requests every hour...");
    
    let mut interval = time::interval(Duration::from_secs(3600)); // 1 hour = 3600 seconds
    
    loop {
        interval.tick().await;


        pulse_monitor().await;
        
       
    }
}



async fn pulse_monitor(){
    let client = reqwest::Client::new();

    //pulse alchemy for the current block ! 
    let network_block =  match get_alchemy_block(&client).await {
        Ok(block_number) => {
            println!("Alchemy current block: {}", block_number);
            Some(block_number)
        }
        Err(e) => {
            eprintln!("Alchemy API failed: {}", e);
            None 
        }
    };

    //hit hasura for the cursor 
    let url = "https://hasura-mainnet.nfteller.org/v1/graphql";
    let cursor_block = match make_post_request(&client, url).await {
        Ok(response) => {
            println!("Hasura GraphQL request successful:");
            println!("{}", response);
            
            // Parse the response to get the cursor block
            match parse_cursor_response(&response) {
                Ok(block_num) => {
                    println!("Cursor block number: {}", block_num);
                    Some(block_num)
                }
                Err(e) => {
                    eprintln!("Failed to parse cursor block: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            eprintln!("Hasura GraphQL request failed: {}", e);
            None 
        }
    };



    println!("{:?}",network_block);
     println!("{:?}",cursor_block);

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

async fn get_alchemy_block(client: &reqwest::Client) -> Result<U256, Box<dyn std::error::Error>> {
    let api_key = env::var("ALCHEMY_API_KEY")
        .map_err(|_| "ALCHEMY_API_KEY environment variable not set")?;
    
    let url = format!("https://eth-mainnet.g.alchemy.com/v2/{}", api_key);
    
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_blockNumber",
        "params": [],
        "id": 1
    });
    
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;
    
    let json: serde_json::Value = response.json().await?;
    
    if let Some(result) = json.get("result") {
        if let Some(block_hex) = result.as_str() {
            // Parse hex string to U256
            let block_number = U256::from_str_radix(block_hex, 16)?;
            return Ok(block_number);
        }
    }
    
    Err("Failed to parse block number from Alchemy response".into())
}

fn parse_cursor_response(response: &str) -> Result<U256, Box<dyn std::error::Error>> {
    let json: serde_json::Value = serde_json::from_str(response)?;
    
    // Navigate to data.cursors array
    if let Some(data) = json.get("data") {
        if let Some(cursors) = data.get("cursors") {
            if let Some(cursors_array) = cursors.as_array() {
                // Find the cursor with the highest block_num
                let mut max_block = U256::zero();
                for cursor in cursors_array {
                    if let Some(block_num) = cursor.get("block_num") {
                        let block_val = if let Some(num) = block_num.as_u64() {
                            U256::from(num)
                        } else if let Some(str_val) = block_num.as_str() {
                            U256::from_dec_str(str_val)?
                        } else {
                            continue;
                        };
                        
                        if block_val > max_block {
                            max_block = block_val;
                        }
                    }
                }
                if max_block > U256::zero() {
                    return Ok(max_block);
                }
            }
        }
    }
    
    Err("No cursors found or invalid response format".into())
}
