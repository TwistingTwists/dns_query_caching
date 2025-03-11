use reqwest::Client;
use std::error::Error;
use std::fs::OpenOptions;
use std::sync::Arc;
use std::time::{Duration, Instant};
use chrono::Utc;
use hickory_resolver::TokioAsyncResolver;
use tracing::{debug, info, instrument};
use tracing_subscriber;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::prelude::*;
use std::net::SocketAddr;

// Custom DNS resolver that wraps hickory-resolver
#[derive(Clone)]
struct HickoryDnsResolver {
    resolver: TokioAsyncResolver,
}

impl HickoryDnsResolver {
    fn new() -> Self {
        // Create custom resolver options with optimized caching
        let mut opts = hickory_resolver::config::ResolverOpts::default();
        opts.cache_size = 1024; // Increase cache size
        opts.use_hosts_file = true;
        opts.timeout = Duration::from_secs(3); // Reduce timeout from default
        opts.attempts = 2; // Reduce retry attempts
        
        let resolver = TokioAsyncResolver::tokio(
            hickory_resolver::config::ResolverConfig::default(),
            opts,
        );
        
        HickoryDnsResolver { resolver }
    }
}

// Custom trait implementation for reqwest DNS resolution
impl reqwest::dns::Resolve for HickoryDnsResolver {
    fn resolve(&self, name: reqwest::dns::Name) -> reqwest::dns::Resolving {
        let resolver = self.resolver.clone();
        let host = name.as_str().to_string();
        
        Box::pin(async move {
            let start = Instant::now();
            debug!("Resolving hostname: {}", host);
            
            match resolver.lookup_ip(host.as_str()).await {
                Ok(lookup) => {
                    let addrs: Vec<SocketAddr> = lookup
                        .iter()
                        .map(|ip| SocketAddr::new(ip, 0))
                        .collect();
                    
                    let duration = start.elapsed();
                    info!("DNS resolution for {} took {:?}", host, duration);
                    debug!("Resolved {} to {} addresses", host, addrs.len());
                    
                    Ok(Box::new(addrs.into_iter()) as Box<dyn Iterator<Item = SocketAddr> + Send>)
                },
                Err(e) => {
                    info!("Failed to resolve {}: {}", host, e);
                    Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("DNS resolution failed: {}", e),
                    )) as Box<dyn Error + Send + Sync>)
                }
            }
        })
    }
}

// Annotate the main function with `instrument` for automatic tracing
#[tokio::main]
#[instrument]
async fn main() -> Result<(), Box<dyn Error>> {
    // Generate a time-based filename (e.g., app-2025-03-11T163000.log)
    let timestamp = Utc::now().format("%Y-%m-%dT%H%M%S").to_string();
    let filename = format!("app-{}.log", timestamp);

    // Open or create the log file
    let log_file = OpenOptions::new()
        .write(true)
        .append(true) // Append to file (though each run gets a new file due to timestamp)
        .create(true) // Create the file if it doesn't exist
        .open(&filename)?;

    // Set up tracing to write to the log file
    let file_layer = Layer::new()
        .with_writer(log_file)
        .with_ansi(false) // Disable colors in file output
        .with_target(true) // Include module/function targets in logs
        .with_line_number(true); // Include line numbers for debugging

    tracing_subscriber::registry()
        .with(file_layer)
        .with(tracing_subscriber::filter::LevelFilter::DEBUG) // Set max level to DEBUG
        .init();

    info!("Starting the application, logging to {}", filename);

    // Create our custom DNS resolver
    let dns_resolver = HickoryDnsResolver::new();
    
    // Build the reqwest client with our custom resolver
    let client = Client::builder()
        .dns_resolver(Arc::new(dns_resolver))
        .timeout(Duration::from_secs(10)) // Overall request timeout
        .build()?;

    debug!("Client built successfully with custom DNS resolver");

    // Define the URL to test DNS caching
    let url = "https://google.com";

    // Make multiple requests to demonstrate caching and measure DNS time
    for i in 1..=5 {  
        info!("Starting request #{}", i);
        let start = Instant::now();
        let response = fetch_url(&client, url).await?;
        let total_time = start.elapsed();
        info!("Request #{} completed with status: {} in {:?}", i, response.status(), total_time);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    info!("All requests completed");
    Ok(())
}

// Separate async function to fetch the URL, instrumented for tracing
#[instrument(fields(url = %url))]
async fn fetch_url(client: &Client, url: &str) -> Result<reqwest::Response, reqwest::Error> {
    debug!("Starting HTTP request to {}", url);

    // The DNS resolution happens inside our custom resolver
    let start = Instant::now();
    let response = client.get(url).send().await?;
    let duration = start.elapsed();

    debug!("Response received");
    info!("Total request time: {:?}", duration);

    Ok(response)
}