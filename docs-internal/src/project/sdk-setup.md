# SDK Setup

Guide for the NullBlock SDK repository.

## Repository

**GitHub**: [github.com/aetherBytes/nullblock-sdk](https://github.com/aetherBytes/nullblock-sdk)
**Documentation**: [aetherbytes.github.io/nullblock-sdk](https://aetherbytes.github.io/nullblock-sdk/)

## Structure

```
nullblock-sdk/
├── docs/                    # Documentation site (GitHub Pages)
│   ├── index.html
│   ├── api/
│   └── guides/
├── sdk/
│   ├── python/             # Python SDK
│   ├── javascript/         # JavaScript/TypeScript SDK
│   └── rust/               # Rust SDK
├── examples/
│   ├── agents/
│   ├── trading/
│   └── defi/
└── README.md
```

## GitHub Pages Setup

1. Go to repository Settings → Pages
2. Source: "Deploy from a branch"
3. Branch: `main`, Folder: `/docs`
4. Save and wait 5-10 minutes

## Publishing Packages

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

## API Access

All SDK methods connect through Erebus (port 3000):

```python
from nullblock import Client

client = Client(api_url="http://localhost:3000")
response = client.agents.chat("Hello HECATE")
```

## Related

- [API Endpoints](../reference/api.md)
- [Architecture Overview](../architecture.md)
