# Unicity Agentic Framework: Neurosymbolic Flow-Based Architecture

An intelligent agentic system that understands natural language queries and autonomously composes 
execution flows. Built with Rust, it uses semantic embeddings to discover and orchestrate tasks from
methods within agents. That means is that an Agents have a bunch of methods 
associated with it which it uses for its own reasons, and allows for emergent new agents and tasks
to be autonomously created by composing existing methods together.

Think: A hyper granular MCP system where each method is a micro-capability that can be
discovered and composed on-demand based on semantic understanding of user intent and AI intent.

Features voice interaction, real-time transaction monitoring, and a modern desktop interface powered by Tauri and React.

NOTE: For now, this only will work on Apple macOS systems due to Tauri dependencies.

## 🧩 Research Context

The Unicity Agentic Framework builds upon six decades of thought linking dataflow programming and cognitive architectures.
It draws direct inspiration from Flow-Based Programming (FBP), first described by J. Paul 
Morrison (1966), which conceptualized software as networks of independent processes connected by data streams.
By fusing this structural paradigm with modern neural-symbolic learning, semantic vector representations, and agentic reasoning, the framework provides a new substrate for autonomous, interpretable computation.

## 🚀 Features

- **🧠 Semantic Query Understanding**: Uses LLM-powered parsing to interpret natural language transaction requests
- **🤖 Autonomous Agent System**: Modular agents (Ping, Swap) that can be dynamically discovered and composed
- **🔍 Vector-based Agent Discovery**: HNSW indexing for efficient semantic search of available agents
- **🎯 Intelligent Flow Composition**: Automatically composes transaction steps based on semantic matching
- **💬 Voice Interface**: Integrated speech-to-text using Whisper for hands-free interaction
- **🖥️ Modern Desktop UI**: React-based frontend with real-time transaction monitoring
- **📊 Transaction Ledger**: Complete transaction history with balance tracking
- **🔧 Native Performance**: Tauri-powered desktop application with Rust backend

## 🛠️ Prerequisites

- **Rust** (latest stable)
- **Node.js** (v18 or higher)
- **npm** or **yarn**
- **API Key** for LLM service (OpenAI-compatible)

## 🏃‍♂️ Quick Start

### Development Mode

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd unicity-agentic-demo
   ```

2. **Set up environment variables**
   
   **Option A: Use .env file (Recommended)**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   # The .env file is automatically loaded when the application starts
   ```
   
   **Option B: Export directly**
   ```bash
   # Required: Your LLM API key
   export API_KEY="your-api-key-here"
   
   # Optional: Custom LLM configuration (defaults to z.ai GLM-4.6)
   export LLM_BASE_URL="https://api.openai.com/v1/chat/completions"
   export LLM_MODEL="gpt-4"
   
   # Optional: Anthropic API key (alternative to API_KEY)
   export ANTHROPIC_API_KEY="your-anthropic-key"
   ```
   
   **Supported LLM Providers:**
   - **OpenAI**: `LLM_BASE_URL="https://api.openai.com/v1/chat/completions"` and `LLM_MODEL="gpt-4"` or `gpt-3.5-turbo`
   - **z.ai**: Default configuration (no changes needed)
   - **Anthropic**: Set `ANTHROPIC_API_KEY` instead of `API_KEY`
   - **Any OpenAI-compatible API**: Set `LLM_BASE_URL` and `LLM_MODEL` accordingly

3. **Run the development script**
   ```bash
   ./run-dev.sh
   ```

   This will automatically:
   - Start the frontend development server
   - Launch the Tauri desktop application
   - Initialize all agents

### Manual Development Setup

If you prefer to run components separately:

1. **Install frontend dependencies**
   ```bash
   cd frontend
   npm install
   ```

2. **Start frontend development server**
   ```bash
   npm run dev
   ```

3. **Start desktop application** (in another terminal)
   ```bash
   cd desktop
   cargo tauri dev
   ```

## 🔨 Building for Production

### Automated Build

```bash
./build-ui.sh
```

### Manual Build

1. **Build frontend**
   ```bash
   cd frontend
   npm run build
   ```

2. **Build desktop application**
   ```bash
   cd desktop
   cargo tauri build
   ```

The built application will be available in `desktop/target/release/bundle/`.

## 💡 Usage Examples

Try these natural language queries in the chat interface:

- **Network Operations**: "Ping the network to check connectivity"
- **Token Swaps**: "Swap 100 USDT for ALPHA"
- **Balance Queries**: "What's my current balance?"
- **Complex Operations**: "Swap 50 USDT to BTC, then check my ETH balance"

The system will:
1. Parse your natural language query
2. Discover relevant agents using semantic search
3. Compose the transaction flow
4. Execute the operations
5. Provide a natural language summary

## 🧩 Architecture

### Neurosymbolic Flow-Based Programming

At the core of Unicity Agentic Demo is a **neurosymbolic architecture** that combines neural language understanding with symbolic execution. This hybrid approach enables the system to:

1. **Understand Natural Language** (Neural): Uses LLM to parse user queries into structured transaction flows
2. **Discover Capabilities Semantically** (Neural): Employs vector embeddings to find relevant agents through semantic similarity
3. **Compose Executable Flows** (Symbolic): Creates deterministic transaction pipelines with type-safe method chaining
4. **Execute with Precision** (Symbolic): Runs composed flows with exact transaction semantics and state management

### Core Components

1. **Agent System**: Modular agents (Ping, Swap) that register their capabilities with semantic hooks
2. **Flow Engine**: Three-stage neurosymbolic processing:
   - **Parser**: Converts natural language to structured transaction flows using LLM
   - **Composer**: Discovers and chains methods using semantic search + LLM approval for ambiguity
   - **Executor**: Executes composed flows with deterministic state management
3. **Vector Index**: HNSW-based semantic search for intelligent agent discovery
4. **Ledger**: Transaction and balance management with precise decimal arithmetic
5. **LLM Integration**: Dual-purpose - query parsing and intelligent method selection

### Neurosymbolic Flow Composition

The system's key innovation is how it bridges neural understanding with symbolic execution:

```
Natural Language Query
        ↓ (Neural Parsing)
Structured Transaction Flow
        ↓ (Semantic Discovery)
Candidate Methods (Vector Search)
        ↓ (LLM Approval if Ambiguous)
Selected Method
        ↓ (Symbolic Chaining)
Executable Flow
        ↓ (Deterministic Execution)
Transaction Results
```

This architecture ensures:
- **Flexibility**: Neural understanding handles varied user expressions
- **Reliability**: Symbolic execution guarantees deterministic results
- **Intelligence**: LLM resolves ambiguity in method selection
- **Scalability**: Vector search enables efficient agent discovery

### Technology Stack

- **Backend**: Rust with async/await for performance and safety
- **Frontend**: React + TypeScript + Vite for modern UI
- **Desktop**: Tauri (Rust + Web frontend) for native experience
- **Database**: SurrealDB (in-memory) for agent and transaction storage
- **AI**: OpenAI-compatible LLM + Whisper for STT
- **Vector Search**: HNSW for efficient semantic matching
- **Neurosymbolic Engine**: Custom flow composition and execution system

## 🔧 Configuration

### Environment Variables

- `API_KEY`: Primary LLM API key (required)
- `LLM_BASE_URL`: LLM API endpoint (optional, defaults to z.ai)
- `LLM_MODEL`: LLM model name (optional, defaults to GLM-4.6)
- `ANTHROPIC_API_KEY`: Anthropic API key (optional alternative to API_KEY)

### Example Configurations

**OpenAI GPT-4:**
```bash
export API_KEY="sk-..."
export LLM_BASE_URL="https://api.openai.com/v1/chat/completions"
export LLM_MODEL="gpt-4"
```

**OpenAI GPT-3.5 Turbo:**
```bash
export API_KEY="sk-..."
export LLM_BASE_URL="https://api.openai.com/v1/chat/completions"
export LLM_MODEL="gpt-3.5-turbo"
```

**z.ai (Default):**
```bash
export API_KEY="your-z-ai-key"
# LLM_BASE_URL and LLM_MODEL use defaults
```

**Custom OpenAI-compatible API:**
```bash
export API_KEY="your-custom-key"
export LLM_BASE_URL="https://your-api-endpoint.com/v1/chat/completions"
export LLM_MODEL="your-custom-model"
```

### Tauri Configuration

The desktop application is configured in `desktop/tauri.conf.json`:
- Frontend development URL: `http://localhost:1420`
- Production build directory: `../frontend/dist`

## 🐛 Troubleshooting

### Common Issues

1. **Frontend not building**: Ensure Node.js dependencies are installed
2. **Tauri build fails**: Check that Rust toolchain is up to date
3. **API errors**: Verify API key is set correctly
4. **Speech recognition not working**: Check microphone permissions

### Debug Mode

Enable debug logging by setting:
```bash
export RUST_LOG=debug
./run-dev.sh
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 📚 Research & Citation

If you use this work in research or academic projects, please cite:

```bibtex
@software{unicity_agentic_framework,
  title={Unicity Agentic Framework: Neurosymbolic Flow-Based Architecture},
  author={Joshua J. Bouw},
  year={2025},
  email={jjb@unicity-labs.com},
  url={https://github.com/unicitynetwork/unicity-agentic-demo},
  note={Integrates neural language understanding with symbolic flow-based programming for autonomous agent composition}
}
```

**APA Citation:**
```
Bouw, J. J. (2025). *Unicity Agentic Framework: Neurosymbolic Flow-Based Architecture* 
[Computer software]. Unicity Labs. GitHub repository. https://github.com/unicitynetwork/unicity-agentic-demo
```

**Key Research Contributions:**
- **Hybrid Neural-Symbolic Architecture**: Novel approach combining LLM understanding with deterministic execution
- **Semantic Agent Discovery**: Vector-based method selection with ambiguity resolution
- **Flow-Based Programming**: Declarative transaction composition with type safety
- **Multi-Modal Interaction**: Natural language, voice, and programmatic interfaces

### 📖 References & Influences
- **Morrison, J. P.** (2010). *Flow-Based Programming: A New Approach to Application Development* (2nd ed.). CreateSpace.  
  Foundational work defining flow-based programming, where independent processes communicate via 
  data streams, the core inspiration for this architecture.
- **Whiting, P. G., & Pascoe, R. C.** (1994). *A History of Data-Flow Languages.* *IEEE Annals of the History of Computing, 16*(4), 38–59.  
  Historical overview tracing dataflow and flow-based paradigms from the 1960s onward.
- **Johnston, W. M., Hanna, J. R., & Millar, R. J.** (2004). *Advances in Dataflow Programming Languages.* *ACM Computing Surveys, 36*(1), 1–34.  
  Summarizes decades of research in dataflow and concurrent programming, linking early concepts to modern distributed systems.
- **Besold, T. R., d’Avila Garcez, A., Bader, S., Bowman, H., Domingos, P., Hitzler, P., and Shanahan, M.** (2017). *Neural-Symbolic Learning and Reasoning: A Survey and Interpretation.* *arXiv:1711.03902.*  
  Comprehensive survey of neural-symbolic integration, forming the theoretical foundation for hybrid cognitive systems.
- **Wang, W., Yang, Y., & Wu, F.** (2022). *Towards Data- and Knowledge-Driven Artificial Intelligence: A Survey on Neural-Symbolic Computing.* *arXiv:2210.15889.*  
  Explains the evolution of neurosymbolic AI and its revival as a bridge between deep learning and logical reasoning.
- **Bouneffouf, D., & Aggarwal, C. C.** (2022). *Survey on Applications of Neurosymbolic Artificial Intelligence.* *arXiv:2209.12618.*  
  Documents emerging real-world uses of neurosymbolic systems, supporting the renewed academic and industrial interest in hybrid AI.
- **DeLong, L. N., Fernández Mir, R., & Fleuriot, J. D.** (2023). *Neurosymbolic AI for Reasoning Over Knowledge Graphs: A Survey.* *arXiv:2302.07200.*  
  Directly relevant to the framework’s semantic-agent discovery and knowledge graph reasoning approach.
- **Odense, S., & d’Avila Garcez, A.** (2022). *A Semantic Framework for Neuro-Symbolic Computing.* *arXiv:2212.12050.*  
  Presents formal semantics for integrating symbolic structures with neural computation, paralleling the “methods as neurons, ports as synapses” model.
- **Marcus, G.** (2020). *The Next Decade in AI: Why Common Sense Is So Hard for Machines.* *Communications of the ACM, 63*(8), 46–53.  
  Argues for the continued need for structured reasoning and hybrid cognitive systems in modern AI.
