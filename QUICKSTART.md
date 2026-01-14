# Quick Start Guide

## Prerequisites
- Rust 1.70+ installed
- SQLite support

## Setup

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd Wazuh-ProcessTree_Rust
   ```

2. **Configure environment**
   ```bash
   cp .env.example .env
   # Edit .env and change the SECRET_KEY for production use
   ```

3. **Build the project**
   ```bash
   cargo build
   ```

4. **Run the server**
   ```bash
   cargo run
   ```

The server will start on `http://127.0.0.1:5443`

## Testing

Access the web interface:
```bash
curl http://127.0.0.1:5443/
```

## Configuration

All configuration is done via environment variables in the `.env` file:

- `HOST`: Server bind address (default: 127.0.0.1)
- `PORT`: Server port (default: 5443)
- `DATABASE_URL`: SQLite database path
- `SECRET_KEY`: Session secret key (min 64 bytes, MUST change in production)
- `LOG_LEVEL`: Logging level (info, debug, warn, error)

## Database

The SQLite database will be created automatically on first run at the path specified in `DATABASE_URL`.
