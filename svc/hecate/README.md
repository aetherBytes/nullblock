# Hecate Frontend

**React-based interface for the NullBlock agent orchestration platform**

## Overview

Hecate is the primary user interface for interacting with NullBlock agents, managing tasks, and exploring the Crossroads marketplace. Built with React, TypeScript, and Vite for a modern, responsive experience.

## Quick Start

### Prerequisites

- Node.js 18+ and npm
- Running Erebus router (port 3000)
- Running Agents service (port 9003)

### Development Setup

```bash
cd svc/hecate
npm install
npm run develop
```

The application will be available at `http://localhost:5173`

## Environment Configuration

Create a `.env` file in the `svc/hecate` directory with the following variables:

```bash
VITE_EREBUS_API_URL=http://localhost:3000
VITE_PROTOCOLS_API_URL=http://localhost:8001
VITE_HECATE_API_URL=http://localhost:9003
VITE_API_GATEWAY=https://randomuser.me/api
VITE_FAST_API_BACKEND_URL=http://localhost:8000
```

**⚠️ IMPORTANT**: All API calls must route through Erebus (port 3000). The `VITE_EREBUS_API_URL` is the primary endpoint for:
- Task management (`/api/agents/tasks`)
- Agent chat (`/api/agents/chat`)
- User registration (`/api/users/register`)
- Wallet operations (`/api/wallets/*`)

**📋 TODO**: Audit and update all hardcoded API URLs in the codebase to use these environment variables consistently.

## **Why Snake Bytes?**

At Snake Bytes, we understand the challenges of keeping up with industry-standard software practices. That's why we've designed a monthly subscription service that brings the expertise of seasoned professionals directly to your virtual doorstep. With our task board-style workflow, inspired by popular tools like Trello and Asana, managing your software tasks has never been easier or more organized.

## **A Spectrum of Technologies at Your Fingertips**

Dive into a pool of possibilities with our diverse technological specializations:
- Python programming at its finest
- Cutting-edge AWS services
- Scalable Microservices and Serverless architectures
- Robust Data pipelines, Web scraping, and ETL processes
- Comprehensive Data analysis and Visualization
- Advanced Machine Learning, Data Science, and Engineering
- State-of-the-art Data Warehousing, Lakes, and Modeling
- Innovative AI Agents, Chatbots, and RAG LLM Agents

## **Customizable Tiers for Every Need**

Whether you're dabbling in data visualization, exploring machine learning, or building complex data engineering projects, Snake Bytes offers various service tiers to match your project's scope and budget. Our goal? To make top-tier software development practices accessible and affordable.

## **Build with the Best**

Our platform harnesses the power of React, Next.Js, FastAPI, and more, ensuring that your projects are built with the best and most suitable technologies. Plus, our task board plugin makes project management a breeze, keeping you on track and in control from start to finish.

## **Join the Revolution**

Step into the future of python software development with Snake Bytes. Our commitment to quality, innovation, and accessibility makes us the ideal partner for your next project. Let's build something extraordinary together.
2. **Install dependencies:**
```bash
npm install
```

3. **Build the WASM module:**
```bash
wasm-pack build
```

4. **Run the development server:**
```bash
npm start
```
Navigate to `http://localhost:3000` to see the app in action!

## 📚 Documentation
Dive deeper into our application with our comprehensive documentation. Learn about the architecture, how to contribute, and more.

## 💡 Contributing
We welcome contributions! Whether it's submitting bugs, requesting new features, or contributing code, our community is open to all. Check out our contributing guidelines for more information.

## 📞 Support
Encountered a problem? Have a question? Our community is here to help. Reach out to us through our support channels or open an issue on GitHub.

## 📃 License
NullBlock is open-sourced under the MIT License. See the LICENSE file for more details.
