# TOON Converter

Super efficient JSON to TOON (and vice versa) converter backend built with Rust, Axum, and gRPC.

## What is TOON?

TOON (Token-Oriented Object Notation) is a modern data format optimized for AI and LLM applications. It uses 30-60% fewer tokens than JSON, making it ideal for AI-driven workflows where token efficiency directly impacts cost and performance.

## Features

- **Blazing Fast**: Built with Rust for maximum performance
- **Dual Protocol Support**: Both gRPC (binary) and REST (HTTP) endpoints
- **Nom Parser**: Uses nom parser combinators for robust, efficient parsing
- **Zero Dependencies**: Minimal footprint, maximum speed
- **Bidirectional Conversion**: JSON to TOON and TOON to JSON

## Architecture

- **Rust + Axum**: High-performance web framework
- **Tonic**: gRPC framework for binary protocol
- **Nom**: Parser combinator library for robust parsing
- **serde_json**: JSON serialization

## API Endpoints

### REST API (Port 5000)

#### Health Check
```bash
GET /
```

#### JSON to TOON
```bash
POST /json-to-toon
Content-Type: application/json

{
  "data": "{\"users\": [{\"id\": 1, \"name\": \"Alice\"}]}"
}
```

Response:
```json
{
  "result": "users[1]{id,name}:\n1,Alice",
  "error": null
}
```

#### TOON to JSON
```bash
POST /toon-to-json
Content-Type: application/json

{
  "data": "users[1]{id,name}:\n1,Alice"
}
```

Response:
```json
{
  "result": "{\n  \"users\": [\n    {\n      \"id\": 1,\n      \"name\": \"Alice\"\n    }\n  ]\n}",
  "error": null
}
```

### gRPC API (Port 50051)

The gRPC service provides the same functionality with binary protocol for maximum efficiency.

Service definition:
```protobuf
service ConverterService {
  rpc JsonToToon (ConvertRequest) returns (ConvertResponse);
  rpc ToonToJson (ConvertRequest) returns (ConvertResponse);
}
```

## Building

```bash
cargo build --release
```

## Running

```bash
./target/release/toon-converter
```

The server will start:
- gRPC server on `0.0.0.0:50051`
- HTTP REST API on `0.0.0.0:5000`

## Example Usage

### Convert JSON to TOON
```bash
curl -X POST http://localhost:5000/json-to-toon \
  -H "Content-Type: application/json" \
  -d '{"data": "{\"users\": [{\"id\": 1, \"name\": \"Sreeni\", \"role\": \"admin\"}]}"}'
```

Output:
```
users[1]{id,name,role}:
1,Sreeni,admin
```

### Convert TOON to JSON
```bash
curl -X POST http://localhost:5000/toon-to-json \
  -H "Content-Type: application/json" \
  -d '{"data": "users[1]{id,name,role}:\n1,Sreeni,admin"}'
```

## TOON Format Specification

TOON uses a compact, tabular format:

```
key[count]{column1,column2,...}:
value1,value2,...
value1,value2,...
```

Example:
```
users[3]{id,name,role,email}:
1,Sreeni,admin,sreeni@example.com
2,Krishna,admin,krishna@example.com
3,Aaron,user,aaron@example.com

metadata{total,last_updated}:
3,2024-01-15T10:30:00Z
```

## Token Efficiency

TOON typically saves 30-60% tokens compared to JSON, making it ideal for:
- LLM API calls
- AI agent communication
- Token-sensitive workflows
- Cost optimization in AI applications

## License

MIT
