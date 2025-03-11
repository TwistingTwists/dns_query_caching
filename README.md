# DNS Caching Experiment

A Rust application that demonstrates DNS caching and resolution performance using custom DNS resolver integration with reqwest.

## Overview

This project showcases:
- Custom DNS resolution with hickory-resolver
- Integration with reqwest HTTP client
- Performance measurement of DNS resolution
- Structured logging with tracing

## Features

- **Custom DNS Resolver**: Implements a custom DNS resolver using hickory-resolver with optimized caching
- **Performance Tracking**: Measures and logs DNS resolution and HTTP request times
- **Structured Logging**: Uses tracing for comprehensive logging with file output

## Requirements

- Rust 2021 edition or later
- Cargo package manager

## Dependencies

- reqwest (with hickory-dns feature)
- tokio
- tracing and tracing-subscriber
- chrono
- hickory-resolver
- futures

## Usage

1. Build the project:
   ```
   cargo build --release
   ```

2. Run the application:
   ```
   cargo run --release
   ```

3. Check the generated log file (named with timestamp, e.g., `app-2025-03-11T163000.log`) for detailed information about DNS resolution and request performance.

## How It Works

The application:
1. Sets up a custom DNS resolver with optimized caching parameters
2. Integrates the resolver with reqwest HTTP client
3. Makes multiple requests to the same URL to demonstrate DNS caching
4. Logs detailed timing information for each request
5. Shows the performance benefits of DNS caching

## Configuration

You can modify the following parameters in the code:
- DNS cache size (currently 1024 entries)
- DNS timeout (currently 3 seconds)
- DNS retry attempts (currently 2)
- HTTP request timeout (currently 10 seconds)
- Target URL for testing (currently "https://travelomatrix.com")

## License

[Add your license information here]
