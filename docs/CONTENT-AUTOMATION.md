# Content Automation Setup

This document explains how to set up automated content generation for Nullblock's social media presence.

---

## 🎯 Overview

**Workflow:**
1. **Cron jobs** generate content automatically based on schedule
2. Content goes into **pending queue** for manual review
3. You review, edit if needed, and post manually
4. Once posted, mark as posted to track what's live

**Philosophy:** Automate the creation, keep human review. The agents suggest, you approve.

---

## 📋 Tools

| Script | Purpose |
|--------|---------|
| `content-generator.js` | Core content generation engine |
| `content-queue.js` | Queue management (generate, list, show, mark posted) |
| `generate-image.sh` | Image generation wrapper (uses nano-banana-pro) |

---

## 🤖 Setting Up Cron Jobs

### Method 1: OpenClaw Cron (Recommended)

Use OpenClaw's built-in cron system to generate content:

```javascript
// Morning Insight - Daily at 9 AM
{
  "name": "Morning Content",
  "schedule": { "kind": "cron", "expr": "0 9 * * *", "tz": "America/Denver" },
  "sessionTarget": "isolated",
  "payload": {
    "kind": "agentTurn",
    "message": "Generate morning insight content: cd /home/sagej/nullblock && node tools/content-queue.js generate MORNING_INSIGHT --image",
    "timeoutSeconds": 120
  }
}

// Progress Update - Daily at 3 PM
{
  "name": "Progress Update",
  "schedule": { "kind": "cron", "expr": "0 15 * * *", "tz": "America/Denver" },
  "sessionTarget": "isolated",
  "payload": {
    "kind": "agentTurn",
    "message": "Generate progress update: cd /home/sagej/nullblock && node tools/content-queue.js generate PROGRESS_UPDATE",
    "timeoutSeconds": 120
  }
}

// Educational - Wednesdays at Noon
{
  "name": "Educational Content",
  "schedule": { "kind": "cron", "expr": "0 12 * * 3", "tz": "America/Denver" },
  "sessionTarget": "isolated",
  "payload": {
    "kind": "agentTurn",
    "message": "Generate educational content: cd /home/sagej/nullblock && node tools/content-queue.js generate EDUCATIONAL --image",
    "timeoutSeconds": 120
  }
}

// Eerie Fun - Sundays at 6 PM
{
  "name": "Eerie Fun Content",
  "schedule": { "kind": "cron", "expr": "0 18 * * 0", "tz": "America/Denver" },
  "sessionTarget": "isolated",
  "payload": {
    "kind": "agentTurn",
    "message": "Generate eerie fun content: cd /home/sagej/nullblock && node tools/content-queue.js generate EERIE_FUN --image",
    "timeoutSeconds": 120
  }
}

// Community - Daily at 6 PM
{
  "name": "Community Engagement",
  "schedule": { "kind": "cron", "expr": "0 18 * * *", "tz": "America/Denver" },
  "sessionTarget": "isolated",
  "payload": {
    "kind": "agentTurn",
    "message": "Generate community content: cd /home/sagej/nullblock && node tools/content-queue.js generate COMMUNITY",
    "timeoutSeconds": 120
  }
}
```

### Adding Cron Jobs via OpenClaw

Use the `cron` tool to add jobs:

```bash
# Add via OpenClaw CLI or tell Mo to add them
openclaw cron add --schedule "0 9 * * *" --task "Generate morning content"
```

Or have Mo add them directly using the `cron` tool.

---

## 📝 Daily Workflow

### 1. Generate Content Manually (Testing)

```bash
cd /home/sagej/nullblock

# Generate content
node tools/content-queue.js generate MORNING_INSIGHT
node tools/content-queue.js generate EERIE_FUN --image

# List pending
node tools/content-queue.js list

# Show specific item
node tools/content-queue.js show MORNING_INSIGHT_1738541342539

# Generate image for item
./tools/generate-image.sh MORNING_INSIGHT_1738541342539
```

### 2. Review & Edit

```bash
# Show full content
node tools/content-queue.js show <ID>

# If you want to edit, copy the text, modify it, and save for manual posting
# Or edit the JSON file directly in content-queue/pending/<ID>.json
```

### 3. Post Manually (via bird)

```bash
# Once bird is set up and authenticated
bird tweet "<your final text>"

# Or post via X web interface manually
```

### 4. Mark as Posted

```bash
# After posting, mark it as done
node tools/content-queue.js mark-posted <ID> https://x.com/nullblock/status/...

# This moves it to the posted archive
```

---

## 🖼️ Image Generation

**Using nano-banana-pro:**

```bash
# Generate image for queued content
./tools/generate-image.sh <CONTENT_ID>

# This will:
# 1. Read the image prompt from the queue item
# 2. Call nano-banana-pro
# 3. Save image to content-queue/images/
# 4. Update queue item with image path
```

**Manual image generation:**

```bash
# Get image prompt
node tools/content-generator.js image-prompt EERIE_FUN

# Use nano-banana-pro directly
nano-banana-pro generate "your prompt here" --output myimage.png
```

---

## 🎬 Video Generation (Future)

**Planned pipeline:**

1. Generate video script
2. Use remotion-video-toolkit to create programmatic video
3. Add to queue with video asset
4. Post to X/YouTube

**For now:**
- Focus on text + image content
- Video can come in Phase 2

---

## 📊 Queue Management

### List pending content
```bash
node tools/content-queue.js list
```

### Show specific item
```bash
node tools/content-queue.js show <ID>
```

### Delete unwanted content
```bash
node tools/content-queue.js delete <ID>
```

### Check what's been posted
```bash
node tools/content-queue.js posted 20
```

---

## 🔄 Automated Posting (Future Phase)

Once you're comfortable with the content quality and want full automation:

1. **Set up bird authentication** (browser cookies or Sweetistics API)
2. **Create posting script** that reads from queue and posts via bird
3. **Add posting cron job** that runs after content generation
4. **Set up monitoring** to alert if posting fails

**Example posting flow:**

```bash
# Posting script (to be created)
node tools/auto-post.js

# This would:
# 1. Read pending queue
# 2. Post via bird
# 3. Mark as posted
# 4. Handle errors gracefully
```

---

## 🛡️ Safety & Review

**Manual review is required when:**
- Content mentions specific people/companies
- Eerie fun content might be misunderstood
- Technical claims need verification
- Anything that could be interpreted as financial advice (never do this)

**Automated posting is safe when:**
- Content has been reviewed and approved
- Templates are well-tested
- Error handling is robust
- You have monitoring in place

---

## 📈 Metrics to Track

Once content is live, track:
- Engagement rate (likes, retweets, replies)
- Which themes perform best
- Best posting times
- Audience growth

This will inform future content strategy adjustments.

---

## 🎭 Content Guidelines (Vault-Tec Energy)

**DO:**
- Be cheerfully inevitable
- Embrace dark humor about AI
- Focus on infrastructure/protocols
- Keep it technical but accessible
- Build excitement about the future
- Show progress transparently

**DON'T:**
- Give financial advice
- Mention specific coins/tokens/trading
- Make unrealistic promises
- Be boring and corporate
- Shy away from the eerie vibe
- Forget the human touch

---

## 🚀 Next Steps

1. **Test the generator** - Generate a few pieces of content manually
2. **Review the vibe** - Make sure it matches Vault-Tec energy
3. **Set up first cron job** - Start with one daily post
4. **Manual posting** - Post a few manually to test
5. **Scale up** - Add more content types as you dial it in
6. **Add bird automation** - Once you're confident in content quality

---

*Created: 2026-02-02*
*Author: Moros (Mo)*
*Status: Ready for testing*
