# Kanshi
A high-performance indexing solution for Starknet blockchain data, written in Rust. This indexer processes Starknet blocks, transactions, and events, storing them in a structured format for efficient querying and analysis.

## Features

- Fast and efficient block processing
- Real-time event indexing
- Support for custom filtering and transformation rules
- Built-in retry mechanism for failed requests
- Comprehensive metrics and monitoring
- REST API for data access

## Prerequisites

- Rust 1.70 or higher
- Starknet node access (RPC endpoint)

## Installation

1. Clone the repository:
```bash
git clone https://github.com/RougeDevs/kanshi
cd kanshi
```

2. Install dependencies:
```bash
cargo build kanshi
```

3. Configure your environment:
```bash
cp .env.example .env
# Edit .env with your configuration
```

## Configuration

The indexer can be configured through environment variables or a config file. Key configuration options include:

- `STARKNET_RPC_URL`: URL of your Starknet node

## Usage

1. Start the indexer:
```bash
cargo run kanshi
```


## API Documentation

## Development

### Running Tests

```bash
cargo test
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## Performance Considerations

- The indexer uses async Rust for optimal performance
- Configurable batch processing for higher throughput
- Connection pooling for database operations
- Caching layer for frequently accessed data

## License

Apache-2.0 License - see [LICENSE](LICENSE) file for details

## Support

For support and questions:
- Open an issue in the GitHub repository
- Check the documentation

---

*Note: This project is under active development. Please report any issues you encounter.*
