# TOON Converter Backend

## Overview
Blazing-fast JSON to TOON converter backend built with Rust, Axum, and gRPC. Provides both REST and gRPC endpoints for bidirectional conversion between JSON and TOON (Token-Oriented Object Notation) format.

## Current State
- Built with Rust using Axum web framework and Tonic gRPC
- Nom parser combinators for robust TOON parsing
- Dual protocol: REST API on port 5000, gRPC on port 50051
- ASCII-only output (no emojis)
- Fully functional bidirectional conversion

## Recent Changes (2025-11-13)
- Implemented complete TOON parser using nom parser combinators
- Added JSON to TOON serializer
- Created dual protocol support (REST + gRPC)
- Removed emojis from all output
- Built in release mode for maximum performance

## Project Architecture

### Core Components
1. **TOON Parser** (`src/toon/parser.rs`) - Nom-based parser for TOON format
2. **TOON Serializer** (`src/toon/serializer.rs`) - Converts JSON to TOON
3. **Converter** (`src/converter.rs`) - Bidirectional conversion logic
4. **Main Server** (`src/main.rs`) - Axum REST + Tonic gRPC servers

### Dependencies
- axum: Web framework
- tonic: gRPC framework
- nom: Parser combinator library
- serde_json: JSON handling
- tokio: Async runtime

## API Endpoints

### REST (Port 5000)
- `GET /` - Health check
- `POST /json-to-toon` - Convert JSON to TOON
- `POST /toon-to-json` - Convert TOON to JSON

### gRPC (Port 50051)
- `JsonToToon` - Convert JSON to TOON
- `ToonToJson` - Convert TOON to JSON

## User Preferences
- ASCII-only output (no emojis)
- JSON to TOON should come before TOON to JSON in all code
- Use nom parser for robust parsing
