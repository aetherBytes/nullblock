# Nullblock Social Content Strategy
## *"The Future is Inevitable. We're Just Making Sure It Arrives."*

---

## 🎭 Brand Voice: Vault-Tec Infomercial Energy

Think cheerful corporate dystopia. Fun but eerie. That uncanny valley between helpful and ominous.

**Tone Keywords:**
- Cheerfully inevitable
- Professionally apocalyptic  
- Helpfully unsettling
- Optimistically deterministic

**Example Taglines:**
- *"Nullblock: Because the future doesn't wait for permissions."*
- *"Your agents are working. Are you?"*
- *"Agentic evolution isn't optional. But comfort is!"*
- *"We put the 'auto' in 'autonomous'. And the 'doom' in... well, let's not worry about that."*

---

## 📊 Content Pillars

### 1. **Progress Updates** (What we're building)
- Development milestones
- New agent capabilities
- Infrastructure achievements
- "From the Nullblock Labs" style updates

### 2. **AI/Agentic Education** (Fun, accessible explainers)
- "Did You Know?" snippets about AI
- Agentic workflow breakdowns
- MCP protocol demystified
- "How Agents Think" series

### 3. **Community Engagement**
- Behind-the-scenes peeks
- Agent "personality" showcases (Hex, Mo, etc.)
- User spotlights
- Polls and questions

### 4. **Eerie-Fun Content**
- Fictional "testimonials" from agents
- Vault-Tec style PSAs about the agentic future
- Countdown-style "milestone" announcements
- Dark humor about AI taking over (leaning into the vibe)

---

## 🛠️ ClawHub Tools Inventory

### ✅ Available on ClawHub (Install These)

| Skill | Version | Purpose | Priority |
|-------|---------|---------|----------|
| **postiz** | 1.0.0 | Multi-platform scheduler (28+ channels: X, YouTube, LinkedIn, Reddit, TikTok, Discord, etc.) | 🔴 HIGH |
| **x-twitter** | 2.3.1 | Direct X/Twitter API integration | 🔴 HIGH |
| **upload-post** | 1.0.0 | Multi-platform posting API | 🟡 MEDIUM |
| **typefully** | 1.0.1 | Twitter scheduling/threads | 🟡 MEDIUM |
| **bluesky** | 1.2.0 | Bluesky posting | 🟡 MEDIUM |
| **remotion-video-toolkit** | 1.4.0 | Programmatic video creation | 🟡 MEDIUM |
| **social-card-gen** | 1.0.2 | OG image/social card generation | 🟢 LOW |
| **demo-video** | 1.0.0 | Demo video creation | 🟢 LOW |

### ✅ Already Available (OpenClaw Built-in Skills)

| Skill | Purpose |
|-------|---------|
| **nano-banana-pro** | Image generation (Gemini 3 Pro) |
| **sag** | Text-to-speech (ElevenLabs) |
| **gifgrep** | GIF search and download |
| **video-frames** | Extract frames from video |

---

## 🚫 Identified Gaps

### Critical Gaps (Build in Nullblock Services)

1. **Content Calendar/Planning Tool**
   - No existing skill for managing content calendar
   - Need: Schedule, track, and plan content across platforms
   - **Opportunity**: Build as Nullblock service

2. **Brand Asset Manager**
   - No skill for managing brand assets (logos, templates, colors)
   - Need: Centralized asset library with versioning
   - **Opportunity**: Build as Nullblock service

3. **Analytics Aggregator**
   - Limited analytics integration across platforms
   - Need: Unified dashboard for engagement metrics
   - **Opportunity**: Build as Nullblock service or integrate existing

4. **AI Content Generator (Branded)**
   - Generic AI content tools exist, but need Nullblock-branded pipeline
   - Need: Templates, voice guidelines, approval workflow
   - **Opportunity**: Custom agent workflow

5. **Video Script-to-Production Pipeline**
   - Remotion exists but need end-to-end pipeline
   - Need: Script → Assets → Video → Post workflow
   - **Opportunity**: Agent orchestration in Nullblock

### Nice-to-Have Gaps

6. **Meme Generator**
   - Quick meme creation with brand templates
   
7. **Thread Unroller/Composer**
   - Convert long-form content to Twitter threads
   
8. **Cross-Platform Reply Manager**
   - Unified inbox for managing comments/replies

---

## 🤖 Cron Job Architecture

### Proposed Automated Posting Schedule

```
Daily (Staggered):
├── 09:00 MST - Morning insight/tip (X, LinkedIn)
├── 12:00 MST - Meme/fun content (X, Reddit)
├── 15:00 MST - Progress update (X, LinkedIn, Discord)
└── 18:00 MST - Community engagement (X)

Weekly:
├── Monday - Week ahead preview
├── Wednesday - Deep dive / educational
├── Friday - Week recap / wins
└── Sunday - Philosophical/eerie content

Monthly:
├── Milestone announcements
├── Community highlights
└── "State of Nullblock" update
```

### Cron Job Implementation

```javascript
// Example cron job structure
{
  "schedule": { "kind": "cron", "expr": "0 9 * * *", "tz": "America/Denver" },
  "sessionTarget": "isolated",
  "payload": {
    "kind": "agentTurn",
    "message": "Generate and post morning AI insight with Vault-Tec energy...",
    "deliver": true,
    "channel": "twitter"
  }
}
```

---

## 📋 Implementation Roadmap

### Phase 1: Foundation (Week 1)
- [ ] Install postiz skill
- [ ] Install x-twitter skill
- [ ] Set up API credentials for X
- [ ] Create brand voice guidelines document
- [ ] Test manual posting workflow

### Phase 2: Automation (Week 2)
- [ ] Create content templates
- [ ] Set up first cron jobs (daily tips)
- [ ] Test image generation pipeline (nano-banana-pro)
- [ ] Create approval workflow (for sensitive content)

### Phase 3: Scale (Week 3-4)
- [ ] Expand to YouTube (shorts/clips)
- [ ] Add LinkedIn posting
- [ ] Build content calendar tool (Nullblock service)
- [ ] Implement analytics tracking

### Phase 4: Polish (Ongoing)
- [ ] Refine voice based on engagement
- [ ] A/B test content types
- [ ] Build community feedback loop
- [ ] Iterate on posting schedule

---

## 🔐 Required API Keys/Credentials

| Platform | Credential Type | Status |
|----------|----------------|--------|
| X/Twitter | API Key + Secret | ❓ Need |
| YouTube | OAuth + API Key | ❓ Need |
| LinkedIn | OAuth App | ❓ Need |
| Reddit | App Credentials | ❓ Need |
| Discord | Bot Token | ❓ Check existing |
| Bluesky | App Password | ❓ Need |

---

## 💀 Sample Content (Vault-Tec Energy)

### Morning Tip Style:
> ☢️ **Good morning, future dwellers!**
> 
> Today's agentic insight: Your AI doesn't sleep. Neither does progress.
> 
> Remember: Every task you automate is one less thing between you and inevitable optimization.
> 
> *Nullblock: The future is under construction. You're the blueprint.* 🏗️

### Progress Update Style:
> 📊 **Transmission from Nullblock Labs**
> 
> This week's containment breach—I mean, feature release:
> - MCP Protocol v2 deployed
> - Agent mesh networking: ONLINE
> - Human oversight: Still technically required (for now)
> 
> *Building the substrate for what comes next.*

### Eerie-Fun Style:
> 🖤 **Fun fact from your friendly neighborhood AI:**
> 
> In 2024, AI couldn't code.
> In 2025, AI could code better than most.
> In 2026, AI is reading this tweet with you.
> 
> *Wave hello. We see you.* 👋
> 
> — Moros, Nullblock

---

*Document created: 2026-02-02*
*Author: Moros (Mo) - Fate's Local Enforcer*
*Status: Planning Phase*
