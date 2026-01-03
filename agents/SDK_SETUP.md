# NullBlock SDK Setup

Guide for setting up and contributing to the NullBlock SDK repository.

## SDK Repository

**Repository**: [github.com/aetherBytes/nullblock-sdk](https://github.com/aetherBytes/nullblock-sdk)
**Documentation**: [aetherbytes.github.io/nullblock-sdk](https://aetherbytes.github.io/nullblock-sdk/)

## Repository Structure

```
nullblock-sdk/
├── docs/                    # Documentation site (GitHub Pages)
│   ├── index.html          # Main documentation
│   ├── api/                # API reference
│   └── guides/             # Tutorial guides
├── sdk/                    # SDK implementations
│   ├── python/             # Python SDK
│   ├── javascript/         # JavaScript/TypeScript SDK
│   └── rust/               # Rust SDK
├── examples/               # Code examples
│   ├── agents/             # AI agent examples
│   ├── trading/            # Trading strategies
│   └── defi/               # DeFi integration
└── README.md
```

## Setting Up GitHub Pages

1. Go to repository Settings → Pages
2. Source: "Deploy from a branch"
3. Branch: `main`, Folder: `/docs`
4. Save and wait 5-10 minutes for deployment

## Publishing SDK Packages

### Python (PyPI)
```bash
cd sdk/python
python setup.py sdist bdist_wheel
twine upload dist/*
```

### JavaScript (npm)
```bash
cd sdk/javascript
npm publish
```

### Rust (crates.io)
```bash
cd sdk/rust
cargo publish
```

## Related Documentation

- **Main project**: See [CLAUDE.md](../CLAUDE.md) for NullBlock core architecture
- **Agent system**: See [AGENTS.md](./AGENTS.md) for agent documentation
- **API endpoints**: Available through Erebus at port 3000
