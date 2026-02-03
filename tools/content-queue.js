#!/usr/bin/env node
/**
 * Content Queue Manager
 * Manages the queue of content ready for manual review and posting
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const QUEUE_DIR = path.join(__dirname, '..', 'content-queue');
const POSTED_DIR = path.join(QUEUE_DIR, 'posted');
const PENDING_DIR = path.join(QUEUE_DIR, 'pending');

// Ensure directories exist
[QUEUE_DIR, POSTED_DIR, PENDING_DIR].forEach(dir => {
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
});

function generateAndQueue(theme, includeImage = false) {
  // Generate content
  const contentJson = execSync(
    `node ${__dirname}/content-generator.js generate ${theme}`,
    { encoding: 'utf-8' }
  );
  const content = JSON.parse(contentJson);
  
  // Generate image prompt if requested
  let imagePrompt = null;
  if (includeImage) {
    imagePrompt = execSync(
      `node ${__dirname}/content-generator.js image-prompt ${theme}`,
      { encoding: 'utf-8' }
    ).trim();
  }
  
  // Create queue item
  const queueItem = {
    id: `${theme}_${Date.now()}`,
    content,
    imagePrompt,
    status: 'pending',
    createdAt: new Date().toISOString(),
    postedAt: null
  };
  
  // Save to pending queue
  const filename = `${queueItem.id}.json`;
  fs.writeFileSync(
    path.join(PENDING_DIR, filename),
    JSON.stringify(queueItem, null, 2)
  );
  
  return queueItem;
}

function listPending() {
  const files = fs.readdirSync(PENDING_DIR).filter(f => f.endsWith('.json'));
  return files.map(f => {
    const data = fs.readFileSync(path.join(PENDING_DIR, f), 'utf-8');
    return JSON.parse(data);
  }).sort((a, b) => new Date(a.createdAt) - new Date(b.createdAt));
}

function listPosted(limit = 10) {
  const files = fs.readdirSync(POSTED_DIR).filter(f => f.endsWith('.json'));
  return files.map(f => {
    const data = fs.readFileSync(path.join(POSTED_DIR, f), 'utf-8');
    return JSON.parse(data);
  })
  .sort((a, b) => new Date(b.postedAt) - new Date(a.postedAt))
  .slice(0, limit);
}

function markAsPosted(id, tweetUrl = null) {
  const filename = `${id}.json`;
  const pendingPath = path.join(PENDING_DIR, filename);
  const postedPath = path.join(POSTED_DIR, filename);
  
  if (!fs.existsSync(pendingPath)) {
    console.error(`Item ${id} not found in pending queue`);
    return null;
  }
  
  const data = JSON.parse(fs.readFileSync(pendingPath, 'utf-8'));
  data.status = 'posted';
  data.postedAt = new Date().toISOString();
  data.tweetUrl = tweetUrl;
  
  fs.writeFileSync(postedPath, JSON.stringify(data, null, 2));
  fs.unlinkSync(pendingPath);
  
  return data;
}

function deleteItem(id) {
  const filename = `${id}.json`;
  const pendingPath = path.join(PENDING_DIR, filename);
  
  if (fs.existsSync(pendingPath)) {
    fs.unlinkSync(pendingPath);
    return true;
  }
  return false;
}

function showItem(id) {
  const filename = `${id}.json`;
  const pendingPath = path.join(PENDING_DIR, filename);
  
  if (!fs.existsSync(pendingPath)) {
    console.error(`Item ${id} not found`);
    return null;
  }
  
  return JSON.parse(fs.readFileSync(pendingPath, 'utf-8'));
}

// CLI interface
const args = process.argv.slice(2);
const command = args[0];

if (command === 'generate') {
  const theme = args[1];
  const includeImage = args.includes('--image');
  
  if (!theme) {
    console.error('Usage: content-queue.js generate <THEME> [--image]');
    process.exit(1);
  }
  
  const item = generateAndQueue(theme, includeImage);
  console.log('Generated and queued:');
  console.log(JSON.stringify(item, null, 2));
  
} else if (command === 'list') {
  const items = listPending();
  console.log(`\n📋 Pending Content (${items.length} items):\n`);
  items.forEach((item, idx) => {
    console.log(`${idx + 1}. [${item.id}]`);
    console.log(`   Theme: ${item.content.theme}`);
    console.log(`   Created: ${new Date(item.createdAt).toLocaleString()}`);
    console.log(`   Text: ${item.content.text.split('\n')[0]}...`);
    console.log(`   Has Image: ${item.imagePrompt ? 'Yes' : 'No'}`);
    console.log('');
  });
  
} else if (command === 'show') {
  const id = args[1];
  if (!id) {
    console.error('Usage: content-queue.js show <ID>');
    process.exit(1);
  }
  
  const item = showItem(id);
  if (item) {
    console.log('\n📄 Content Item:\n');
    console.log('ID:', item.id);
    console.log('Theme:', item.content.theme);
    console.log('Created:', new Date(item.createdAt).toLocaleString());
    console.log('\nText:');
    console.log('─'.repeat(50));
    console.log(item.content.text);
    console.log('─'.repeat(50));
    console.log('\nTags:', item.content.tags.join(' '));
    if (item.imagePrompt) {
      console.log('\nImage Prompt:');
      console.log(item.imagePrompt);
    }
  }
  
} else if (command === 'posted') {
  const limit = parseInt(args[1]) || 10;
  const items = listPosted(limit);
  console.log(`\n✅ Recently Posted (${items.length} items):\n`);
  items.forEach((item, idx) => {
    console.log(`${idx + 1}. [${item.id}]`);
    console.log(`   Posted: ${new Date(item.postedAt).toLocaleString()}`);
    console.log(`   URL: ${item.tweetUrl || 'N/A'}`);
    console.log('');
  });
  
} else if (command === 'mark-posted') {
  const id = args[1];
  const url = args[2];
  
  if (!id) {
    console.error('Usage: content-queue.js mark-posted <ID> [URL]');
    process.exit(1);
  }
  
  const item = markAsPosted(id, url);
  if (item) {
    console.log(`✅ Marked as posted: ${id}`);
  }
  
} else if (command === 'delete') {
  const id = args[1];
  
  if (!id) {
    console.error('Usage: content-queue.js delete <ID>');
    process.exit(1);
  }
  
  if (deleteItem(id)) {
    console.log(`🗑️  Deleted: ${id}`);
  } else {
    console.error(`Item ${id} not found`);
  }
  
} else {
  console.log('Content Queue Manager');
  console.log('');
  console.log('Usage:');
  console.log('  generate <THEME> [--image]  Generate and queue content');
  console.log('  list                        List pending content');
  console.log('  show <ID>                   Show full content item');
  console.log('  posted [limit]              List recently posted items');
  console.log('  mark-posted <ID> [URL]      Mark item as posted');
  console.log('  delete <ID>                 Delete pending item');
  console.log('');
  console.log('Examples:');
  console.log('  content-queue.js generate MORNING_INSIGHT');
  console.log('  content-queue.js generate EERIE_FUN --image');
  console.log('  content-queue.js list');
  console.log('  content-queue.js show MORNING_INSIGHT_1738541342539');
  console.log('  content-queue.js mark-posted MORNING_INSIGHT_1738541342539 https://x.com/...');
}
