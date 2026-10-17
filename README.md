# Anydex

[日本語](./README.ja.md)

> **Bring Your Own [ANY].**
> Choose your files, extraction methods, embedding models, and vector database to build the search platform that works for you.

Anydex is a desktop application for collecting and searching files from a variety of sources. It supports semantic search across text and images, and includes an MCP server that AI agents can use.

The idea is to get started with sensible defaults, then adapt the setup to your environment as your needs grow.

> **In development:** Anydex is currently under active development. Features and configuration may change.

## Bring Your Own [ANY]

Like BYOK (Bring Your Own Key), Anydex lets you choose the components of your search pipeline. The default implementations cover a wide range of use cases, and you can configure or replace individual components when needed.

| Extension point    | Role                                                                  | Default implementation                                                   |
| ------------------ | --------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| **File Source**    | Where files come from; choose services and connection settings        | OpenDAL-based, supporting various services including Google Drive        |
| **File Extractor** | Extracts searchable text and images from files                        | Extraction from PDF, Office, HTML, and other formats; OCR; text chunking |
| **Embedder**       | Converts text and images into vectors for search                      | Local embeddings with Qwen3-VL                                           |
| **Vector Store**   | Stores file information and vectors, and performs similarity searches | LanceDB                                                                  |

Plugin settings are rendered as forms from schemas and can be configured in the app. You can also test embedder connections and file sources. Custom plugins can be implemented in Rust and connected through the Anydex Plugin API.

File sources that require authentication can use OAuth 2.0 PKCE. You can configure a client ID, authorization and token endpoints, scopes, and other settings to register an OAuth client for your service.

## Search your files with AI agents

Anydex can host an MCP server. Select and expose an index to let MCP-compatible AI agents and clients use its file search capabilities.

Available MCP tools:

- List exposed indexes
- List directory contents
- Search text by path
- Perform semantic searches using text or images
- Retrieve file information and download files

Start the server in the app, then connect your MCP client to `http://localhost:8080/mcp`. You can change the port in the app.

> **Network note:** The MCP server listens on the selected port. Depending on your environment, other devices on the same network may also be able to connect. Take care when choosing which indexes to expose and which port to use.

## Features

- **Search across your files:** Create indexes by combining data sources, extractors, embedding models, and storage
- **Search by meaning:** In addition to keyword search, use text or images for semantic search
- **Start locally:** The default embedding model runs locally, so you can try Anydex without setting up an embedding API
- **Schedule indexing:** Set a schedule to index files
- **Connect to AI:** Search through your usual MCP-compatible clients using MCP
- **Manage everything on your desktop:** Manage indexes, OAuth, running tasks, and search results in one app

## Data flow

```text
File source
     │
     ▼
File extraction ──► Embedding ──► Vector store
                                    │
                     ┌──────────────┴──────────────┐
                     ▼                             ▼
              Anydex search UI               MCP server
                                                   │
                                                   ▼
                                              AI agent
```

## Development

Requirements: Node.js, the Rust toolchain, `cargo-about`, and OS-specific dependencies required by Tauri. Install `cargo-about` with `cargo install --locked --features cli cargo-about`.

Build the default plugins, then start the app:

```bash
cd src-rs/src-plugin/anydex-plugin-default-impl
cargo xtask build
cd ../../..
npm install
npm run tauri:dev
```

To start only the frontend:

```bash
npm run dev
```

To build a distributable:

```bash
npm run tauri:build
```

License information for dependency libraries shown on the in-app "Licenses" page is collected automatically when you run `npm run dev` or `npm run build`. This covers npm production dependencies and Cargo dependencies in the Tauri application and plugin workspace.

The Anydex application and Rust plugin source code are available under the [MIT License](./LICENSE).

For instructions on building for CUDA / Metal, see the [default plugins README](./src-rs/src-plugin/anydex-plugin-default-impl/README.md). To implement a custom plugin, see the [Anydex Plugin API](./src-rs/src-plugin/anydex-plugin-api/).
