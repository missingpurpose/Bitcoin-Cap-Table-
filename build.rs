use anyhow::{Ok, Result};
use reqwest::blocking::Client;
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), anyhow::Error> {
    // Define the endpoint URL
    let url = "https://bitcoin.firehose.pinax.network:443";

    // Create a new HTTP client
    let client = Client::new();

    // Send a GET request to the endpoint
    let response = client.get(url).send()?;

    // Check if the request was successful
    if response.status().is_success() {
        // Get the response text
        let data = response.text()?;

        // Write the data to a file
        let mut file = File::create("src/data/bitcoin_data.json")?;
        file.write_all(data.as_bytes())?;
    } else {
        // Handle the error
        eprintln!("Failed to fetch data: {}", response.status());
    }

    Ok(())
}