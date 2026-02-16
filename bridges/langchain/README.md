# LangChain/CrewAI Bridge

This directory contains auto-generated LangChain tool definitions for self-authorship and reflection workflows on Selemene Engine.

## Generating Tools

Run the generator from the project root:

```bash
./scripts/generate-langchain-tools.py
```

This will read `openapi-unified.json` and generate `selemene_tools.py` with:
- Pydantic input schemas for each endpoint
- Tool functions that call the Selemene API
- LangChain `StructuredTool` instances
- `get_all_tools()` function to retrieve all tools

## Usage

```python
from bridges.langchain.selemene_tools import get_all_tools

# Get all Selemene tools
tools = get_all_tools()

# Use with LangChain agents
from langchain.agents import initialize_agent, AgentType
from langchain_openai import ChatOpenAI

llm = ChatOpenAI(model="gpt-4")
agent = initialize_agent(
    tools=tools,
    llm=llm,
    agent=AgentType.STRUCTURED_CHAT_ZERO_SHOT_REACT_DESCRIPTION,
    verbose=True
)

# Use with CrewAI
from crewai import Agent, Task, Crew

agent = Agent(
    role="Vedic Astrology Analyst",
    goal="Calculate and interpret Vedic charts",
    tools=tools,
)
```

## Environment Variables

- `SELEMENE_RUST_URL` - Base URL for Rust engines (default: http://localhost:8080)
- `SELEMENE_TS_URL` - Base URL for TypeScript engines (default: http://localhost:3001)
- `SELEMENE_API_KEY` - Optional API key for authentication

## Dependencies

```bash
pip install langchain-core pydantic httpx
```
