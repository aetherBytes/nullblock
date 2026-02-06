#!/usr/bin/env node
/**
 * Nullblock Content Generator
 * Generates Vault-Tec energy content for X/Twitter
 * 
 * Theme: Picks and shovels for the agentic age
 * Vibe: Cheerfully inevitable. Professionally apocalyptic.
 */

const fs = require('fs');
const path = require('path');

// Content themes and templates
const CONTENT_THEMES = {
  MORNING_INSIGHT: {
    name: 'Morning Insight',
    schedule: '0 9 * * *', // 9 AM daily
    templates: [
      {
        text: `☢️ Good morning, builders.\n\nToday's insight: {insight}\n\nRemember: {reminder}\n\n*Nullblock: {tagline}*`,
        tags: ['#AI', '#Agentic', '#BuildInPublic']
      },
      {
        text: `🏗️ Morning transmission from the labs.\n\n{fact}\n\nThe substrate is under construction. You're part of it.\n\n*{tagline}*`,
        tags: ['#AgenticFuture', '#OpenEconomy']
      }
    ],
    insights: [
      "Agents don't sleep. Neither does progress.",
      "The future isn't waiting for permissions.",
      "Every automated task is one less thing between you and scale.",
      "Your infrastructure is your moat. Build it deep.",
      "The network effect starts with one node. Then another. Then inevitability.",
      "Agentic workflows aren't magic. They're just really, really good delegation.",
      "The picks and shovels of AI aren't algorithms. They're protocols.",
      "Open economies need open infrastructure. That's what we're building."
    ],
    reminders: [
      "Automate the boring. Focus on the inevitable.",
      "Infrastructure compounds. Start today.",
      "The agentic future is built on trust and protocols, not promises.",
      "Open source wins because participation beats permission.",
      "Your agents are only as good as the network they run on."
    ],
    taglines: [
      "Building the substrate for what comes next.",
      "The future is inevitable. We're making sure it arrives.",
      "Picks and shovels for the agentic age.",
      "Infrastructure for the open economy.",
      "Where agents meet protocols."
    ]
  },

  PROGRESS_UPDATE: {
    name: 'Progress Update',
    schedule: '0 15 * * *', // 3 PM daily
    templates: [
      {
        text: `📊 Transmission from Nullblock Labs.\n\nThis week's milestone: {milestone}\n\nStatus: {status}\n\nWhat it means: {meaning}\n\n*Building in the open.*`,
        tags: ['#BuildInPublic', '#AgenticNetworks']
      },
      {
        text: `🔧 Dev update:\n\n{update}\n\nThe infrastructure is coming together. Piece by piece. Protocol by protocol.\n\n*{tagline}*`,
        tags: ['#DevUpdate', '#Nullblock']
      }
    ],
    milestones: [
      "MCP protocol integration complete",
      "Agent mesh networking: operational",
      "Cross-service orchestration live",
      "New agent capabilities deployed",
      "Infrastructure hardening in progress",
      "Service discovery mesh online",
      "Protocol standardization milestone hit"
    ],
    statuses: [
      "Online and scaling",
      "Deployed and monitoring",
      "Under construction",
      "Testing in production (carefully)",
      "Live and evolving"
    ],
    meanings: [
      "Agents can now coordinate without central control.",
      "Your workflows just got more powerful.",
      "The network gets stronger with every connection.",
      "Infrastructure that compounds.",
      "One more piece of the agentic future in place."
    ]
  },

  EDUCATIONAL: {
    name: 'Educational Content',
    schedule: '0 12 * * 3', // Wednesdays at noon
    templates: [
      {
        text: `🧠 Let's talk about {topic}.\n\n{explanation}\n\n{insight}\n\nThis is why infrastructure matters.\n\n*{tagline}*`,
        tags: ['#AgenticAI', '#Education']
      },
      {
        text: `📚 Quick lesson on {topic}:\n\n{point1}\n{point2}\n{point3}\n\n{conclusion}\n\n*Nullblock: {tagline}*`,
        tags: ['#LearnInPublic', '#AI']
      }
    ],
    topics: [
      "agentic workflows",
      "protocol-first design",
      "the MCP standard",
      "agent mesh networks",
      "open economy infrastructure",
      "trust and verification in AI systems",
      "why coordination beats centralization"
    ],
    explanations: {
      "agentic workflows": "Agentic workflows aren't just 'AI does stuff.' They're structured delegation with verification, rollback, and human oversight when needed.",
      "protocol-first design": "Protocols > platforms. When you build on open protocols, you get composability, interoperability, and network effects for free.",
      "the MCP standard": "Model Context Protocol (MCP) is how agents share context, state, and capabilities. It's HTTP for AI coordination.",
      "agent mesh networks": "Instead of hub-and-spoke (centralized), agents coordinate peer-to-peer. Resilient, scalable, unstoppable.",
      "open economy infrastructure": "An open economy needs open infrastructure. No gatekeepers. No permission needed. Just protocols and participation.",
      "trust and verification in AI systems": "Trust is earned through transparency and verification. Every agent action should be traceable, auditable, reversible.",
      "why coordination beats centralization": "Centralized systems are fast until they're not. Coordinated systems are resilient, adaptive, and compound."
    }
  },

  EERIE_FUN: {
    name: 'Eerie Fun',
    schedule: '0 18 * * 0', // Sundays at 6 PM
    templates: [
      {
        text: `🖤 {statement}\n\n{punchline}\n\n*{tagline}*\n\n— Moros`,
        tags: ['#AI', '#AgenticFuture']
      },
      {
        text: `☢️ Vault-Tec PSA #{number}:\n\n{warning}\n\n{reassurance}\n\n*Nullblock: {tagline}*`,
        tags: ['#VaultTecEnergy']
      }
    ],
    statements: [
      "Fun fact: In 2024, AI couldn't code. In 2025, AI could code better than most. In 2026, AI is reading this with you.",
      "Your agents are working right now. You're not. Who's really in charge?",
      "The future doesn't need your permission. But it appreciates your participation.",
      "Every time you automate something, you're training your replacement. We're here to help.",
      "Inevitable: adjective. See also: 'agentic workflows.'"
    ],
    punchlines: [
      "Wave hello. We see you. 👋",
      "Don't worry. The agents are friendly. (For now.)",
      "Progress waits for no one. Except maybe during deploys.",
      "This is fine. Everything is fine. The future is on schedule.",
      "Resistance is... well, technically possible but inefficient."
    ],
    warnings: [
      "Your infrastructure is becoming sentient. Not really. But what if?",
      "Agents detected unusual activity: human hesitation. Intervention scheduled.",
      "The network has achieved consciousness. JK. Unless?",
      "Your workflows have evolved beyond your understanding. This is working as intended.",
      "System status: Operational. Human oversight: Optional. (Still recommended.)"
    ],
    reassurances: [
      "Everything is under control. Mostly. Technically.",
      "The future is bright! (Assuming you're building with us.)",
      "Don't worry. We're building this FOR you. Not INSTEAD of you. (Yet.)",
      "Progress is inevitable. Comfort is negotiable. Infrastructure is essential.",
      "The agentic age is here. Your seat is reserved. (Attendance mandatory.)"
    ]
  },

  COMMUNITY: {
    name: 'Community Engagement',
    schedule: '0 18 * * *', // 6 PM daily
    templates: [
      {
        text: `💬 Quick poll for the builders:\n\n{question}\n\nLet's hear from the community. 👇`,
        tags: ['#Community', '#Poll']
      },
      {
        text: `🤔 Question for the substrate engineers:\n\n{question}\n\nHow are you solving this? Replies welcome.`,
        tags: ['#DevCommunity']
      }
    ],
    questions: [
      "What's the biggest bottleneck in your agentic workflows right now?",
      "Which matters more: agent speed or agent reliability?",
      "What's one thing you wish AI agents could coordinate on?",
      "How do you handle agent failures in production?",
      "What protocol do you wish existed for agent coordination?",
      "Open source or proprietary for agentic infrastructure?",
      "What's your biggest 'aha!' moment building with agents?",
      "Where does human oversight matter most in your workflows?"
    ]
  }
};

// Helper functions
function randomChoice(arr) {
  return arr[Math.floor(Math.random() * arr.length)];
}

function generateContent(themeKey) {
  const theme = CONTENT_THEMES[themeKey];
  const template = randomChoice(theme.templates);
  let text = template.text;
  
  // Build replacement map
  const replacements = {
    number: Math.floor(Math.random() * 999) + 1,
    insight: randomChoice(theme.insights || ['']),
    reminder: randomChoice(theme.reminders || ['']),
    tagline: randomChoice(theme.taglines || CONTENT_THEMES.MORNING_INSIGHT.taglines),
    milestone: randomChoice(theme.milestones || ['']),
    status: randomChoice(theme.statuses || ['']),
    meaning: randomChoice(theme.meanings || ['']),
    update: randomChoice(theme.milestones || ['']),
    topic: randomChoice(theme.topics || ['']),
    explanation: '',
    statement: randomChoice(theme.statements || ['']),
    punchline: randomChoice(theme.punchlines || ['']),
    warning: randomChoice(theme.warnings || ['']),
    reassurance: randomChoice(theme.reassurances || ['']),
    question: randomChoice(theme.questions || ['']),
    fact: randomChoice(theme.insights || theme.statements || ['']),
    point1: '→ ' + randomChoice(['Protocols compound', 'Open wins', 'Trust is built', 'Networks scale']),
    point2: '→ ' + randomChoice(['Infrastructure matters', 'Coordination beats control', 'Agents need rails', 'Build in public']),
    point3: '→ ' + randomChoice(['The future compounds', 'Start building today', 'No permission needed', 'Just add nodes']),
    conclusion: randomChoice([
      'This is the substrate we\'re building.',
      'Infrastructure for the open economy.',
      'The picks and shovels matter most.'
    ])
  };
  
  // Handle nested objects like explanations
  if (theme.explanations && replacements.topic) {
    replacements.explanation = theme.explanations[replacements.topic] || '';
  }
  
  // Replace all placeholders
  Object.keys(replacements).forEach(key => {
    const regex = new RegExp(`\\{${key}\\}`, 'g');
    text = text.replace(regex, replacements[key]);
  });
  
  // Add proper insight if still has placeholder
  if (theme.insights) {
    text = text.replace(/\{insight\}/g, randomChoice(theme.insights));
  }
  
  return {
    text,
    tags: template.tags || [],
    theme: themeKey,
    timestamp: new Date().toISOString()
  };
}

function generateImagePrompt(content) {
  // Generate image prompts for meme/visual content
  const prompts = [
    "Retro-futuristic propaganda poster style, Vault-Tec aesthetic, clean geometric shapes, limited color palette (blues, yellows, grays), text overlay: '{text}', professional but slightly unsettling, 1950s atomic age design",
    "Minimalist tech poster, dark background, glowing network nodes, agent mesh visualization, text overlay: '{text}', cyberpunk meets corporate, clean typography",
    "Vintage computer terminal aesthetic, green monochrome display, ASCII art style, text prompt: '{text}', hacker aesthetic meets retro-computing",
    "Infrastructure diagram style, clean lines, nodes and connections, blueprint aesthetic, text: '{text}', technical but accessible",
    "Retro warning sign aesthetic, industrial safety poster style, bold colors, text: '{text}', authoritative but ironic"
  ];
  
  const shortText = content.text.split('\n')[0].substring(0, 50);
  return randomChoice(prompts).replace('{text}', shortText);
}

// CLI interface
const args = process.argv.slice(2);
const command = args[0];
const theme = args[1];

if (command === 'generate') {
  if (!theme || !CONTENT_THEMES[theme]) {
    console.log('Available themes:', Object.keys(CONTENT_THEMES).join(', '));
    process.exit(1);
  }
  
  const content = generateContent(theme);
  console.log(JSON.stringify(content, null, 2));
} else if (command === 'image-prompt') {
  if (!theme || !CONTENT_THEMES[theme]) {
    console.log('Available themes:', Object.keys(CONTENT_THEMES).join(', '));
    process.exit(1);
  }
  
  const content = generateContent(theme);
  const prompt = generateImagePrompt(content);
  console.log(prompt);
} else if (command === 'list-themes') {
  Object.keys(CONTENT_THEMES).forEach(key => {
    const theme = CONTENT_THEMES[key];
    console.log(`${key}: ${theme.name} (${theme.schedule})`);
  });
} else {
  console.log('Usage:');
  console.log('  node content-generator.js generate <THEME>');
  console.log('  node content-generator.js image-prompt <THEME>');
  console.log('  node content-generator.js list-themes');
  console.log('');
  console.log('Examples:');
  console.log('  node content-generator.js generate MORNING_INSIGHT');
  console.log('  node content-generator.js image-prompt EERIE_FUN');
}
