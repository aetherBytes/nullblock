# Nullblock SDK Repository Structure

This file contains the complete structure that should be created in the **nullblock-sdk** repository.

## 📁 Required Directory Structure

```
nullblock-sdk/
├── docs/                    # Documentation (GitHub Pages root)
│   ├── index.html          # Main documentation site
│   ├── _config.yml         # Jekyll configuration
│   ├── Gemfile             # Ruby dependencies
│   ├── api/                # API documentation
│   │   └── index.md
│   ├── guides/             # Tutorial guides
│   │   ├── getting-started.md
│   │   ├── architecture.md
│   │   ├── agents.md
│   │   ├── trading.md
│   │   └── development.md
│   └── assets/             # Images, CSS, JS
├── sdk/                    # SDK packages
│   ├── python/             # Python SDK
│   │   ├── README.md
│   │   ├── setup.py
│   │   ├── pyproject.toml
│   │   └── nullblock/
│   ├── javascript/         # JavaScript/TypeScript SDK
│   │   ├── README.md
│   │   ├── package.json
│   │   ├── tsconfig.json
│   │   └── src/
│   └── rust/               # Rust SDK
│       ├── README.md
│       ├── Cargo.toml
│       └── src/
├── examples/               # Code examples
│   ├── agents/             # AI agent examples
│   │   ├── simple_agent.py
│   │   ├── arbitrage_agent.py
│   │   └── social_trading_agent.py
│   ├── trading/            # Trading strategies
│   │   ├── basic_trading.py
│   │   ├── basic_trading.js
│   │   ├── arbitrage_bot.py
│   │   ├── portfolio_tracker.py
│   │   └── react_component.jsx
│   └── defi/               # DeFi integration
│       ├── yield_farming.py
│       ├── liquidity_provision.py
│       └── flash_loans.py
├── packages/               # Published packages (optional)
│   ├── nullblock-sdk-py/   # Python package
│   ├── nullblock-sdk-js/   # JavaScript package
│   └── nullblock-sdk-rs/   # Rust package
├── .github/                # GitHub workflows
│   └── workflows/
│       └── pages.yml       # GitHub Pages deployment
├── README.md               # Main entry point
├── CONTRIBUTING.md         # Contribution guidelines
├── LICENSE                 # License file
└── CHANGELOG.md            # Version history
```

## 🚀 GitHub Pages Setup

1. **Enable GitHub Pages** in the nullblock-sdk repository:
   - Go to Settings → Pages
   - Source: "Deploy from a branch"
   - Branch: `main`
   - Folder: `/docs`

2. **Jekyll Configuration** (_config.yml):
```yaml
title: Nullblock SDK Documentation
description: Comprehensive documentation for Nullblock SDKs and APIs
baseurl: "/nullblock-sdk"
url: "https://aetherbytes.github.io"

# Build settings
markdown: kramdown
highlighter: rouge
permalink: pretty

# Theme settings
theme: jekyll-theme-cayman
```

3. **Documentation URL**: `https://aetherbytes.github.io/nullblock-sdk/`

## 📋 Files to Copy

Copy the following files from this repository to the nullblock-sdk repository:

### Documentation Files
- `/docs/index.html` → `nullblock-sdk/docs/index.html`
- `/docs/_config.yml` → `nullblock-sdk/docs/_config.yml` (update baseurl)
- `/docs/Gemfile` → `nullblock-sdk/docs/Gemfile`
- `/docs/index.md` → `nullblock-sdk/docs/guides/getting-started.md`
- `/docs/api.md` → `nullblock-sdk/docs/api/index.md`
- `/docs/getting-started.md` → `nullblock-sdk/docs/guides/getting-started.md`

### SDK Files
- `/sdk/python/README.md` → `nullblock-sdk/sdk/python/README.md`
- `/sdk/javascript/README.md` → `nullblock-sdk/sdk/javascript/README.md`

### Repository Files
- Create new `nullblock-sdk/README.md` (see template below)
- Create `nullblock-sdk/CONTRIBUTING.md`
- Create `nullblock-sdk/LICENSE`

## 📝 Updated URLs

All documentation will be available at:
- **Main Docs**: https://aetherbytes.github.io/nullblock-sdk/
- **API Reference**: https://aetherbytes.github.io/nullblock-sdk/api/
- **Getting Started**: https://aetherbytes.github.io/nullblock-sdk/guides/getting-started/
- **Examples**: https://github.com/aetherBytes/nullblock-sdk/tree/main/examples/

## ⚡ Quick Setup Commands

After creating the nullblock-sdk repository:

```bash
# Clone the new repository
git clone https://github.com/aetherBytes/nullblock-sdk.git
cd nullblock-sdk

# Create directory structure
mkdir -p docs/{api,guides,assets}
mkdir -p sdk/{python,javascript,rust}
mkdir -p examples/{agents,trading,defi}
mkdir -p packages/{nullblock-sdk-py,nullblock-sdk-js,nullblock-sdk-rs}
mkdir -p .github/workflows

# Copy files from this repository
# (Copy the documentation files as listed above)

# Initialize and push
git add .
git commit -m "Initial SDK repository setup with documentation"
git push origin main
```

This structure will provide a clean, professional SDK repository with proper documentation hosting via GitHub Pages.