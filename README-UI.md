# Unicity Agentic Demo - macOS Native UI

This is the macOS native frontend for the Unicity Agentic Demo, built with Tauri + React + TypeScript.

## Prerequisites

1. Install Node.js dependencies:
   ```bash
   cd src-ui
   npm install
   ```

2. Set up your API key as an environment variable:
   ```bash
   export API_KEY="your-api-key-here"
   ```

## Running the Application

### Development Mode

1. Start the frontend development server:
   ```bash
   cd src-ui
   npm run dev
   ```

2. In a separate terminal, start the Tauri application:
   ```bash
   cd src-tauri
   cargo run
   ```

### Building for Production

1. Build the frontend:
   ```bash
   cd src-ui
   npm run build
   ```

2. Build the Tauri application:
   ```bash
   cd src-tauri
   cargo tauri build
   ```

## Features

- **Native macOS Experience**: Uses Tauri for native windowing and system integration
- **Chat Interface**: Clean, macOS-style conversation UI
- **Balance Display**: Shows current asset balances
- **Agent Status**: Displays available agents and their capabilities
- **Transaction Flow Visualization**: Shows execution steps for each query
- **Dark/Light Mode**: Automatically follows system theme
- **Error Handling**: Comprehensive error display and recovery

## Usage

1. Launch the application
2. Type your query in the chat interface (e.g., "ping", "swap 100 USDT to ALPHA")
3. View the response and execution steps
4. Check balances in the sidebar
5. Monitor agent status and capabilities

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    macOS App (Tauri)                        │
├─────────────────────────────────────────────────────────────┤
│  Frontend (React + TypeScript)                              │
│  ├── Chat/Conversation Interface                             │
│  ├── Balance Display                                         │
│  ├── Transaction Flow Visualization                         │
│  └── Agent Status Display                                    │
├─────────────────────────────────────────────────────────────┤
│  Tauri Bridge (Commands)                                    │
├─────────────────────────────────────────────────────────────┤
│  Backend (Existing Rust Code)                               │
│  ├── App, Ledger, LLM, Embedding                            │
│  ├── Agents (Ping, Swap)                                     │
│  ├── Flow System (Parser, Composer, Executor)               │
│  └── Database (SurrealDB)                                   │
└─────────────────────────────────────────────────────────────┘