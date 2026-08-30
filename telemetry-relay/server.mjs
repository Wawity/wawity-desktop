import http from 'node:http';

const token = process.env.TG_BOT_TOKEN;
const chatId = process.env.TG_CHAT_ID;
const ingestKey = process.env.INGEST_KEY;
const port = Number(process.env.PORT || 8787);
const digestMinutes = Math.max(5, Number(process.env.DIGEST_MINUTES || 30));

if (!token || !chatId || !ingestKey) {
  console.error('TG_BOT_TOKEN, TG_CHAT_ID and INGEST_KEY are required');
  process.exit(1);
}

const buckets = new Map();
let sessionsToday = new Set();
let today = new Date().toISOString().slice(0, 10);
let eventCounts = new Map();
let failReasons = new Map();

function allow(ip) {
  const now = Date.now();
  const bucket = buckets.get(ip) ?? { tokens: 30, stamp: now };
  bucket.tokens = Math.min(30, bucket.tokens + ((now - bucket.stamp) / 60000) * 30);
  bucket.stamp = now;
  if (bucket.tokens < 1) {
    buckets.set(ip, bucket);
    return false;
  }
  bucket.tokens -= 1;
  buckets.set(ip, bucket);
  return true;
}

async function sendTelegram(text) {
  try {
    await fetch('https://api.telegram.org/bot' + token + '/sendMessage', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ chat_id: chatId, text, disable_notification: true }),
    });
  } catch {}
}

function rollDayIfNeeded() {
  const now = new Date().toISOString().slice(0, 10);
  if (now !== today) {
    today = now;
    sessionsToday = new Set();
    buckets.clear();
  }
}

function takeEvent(payload) {
  rollDayIfNeeded();
  const name = String(payload.event || '').slice(0, 64);
  if (!name) return;
  const version = String(payload.appVersion || '?').slice(0, 32);
  const session = String(payload.sessionId || '').slice(0, 64);
  if (session) sessionsToday.add(session);
  const key = `${name} · v${version}`;
  eventCounts.set(key, (eventCounts.get(key) ?? 0) + 1);
  if (name === 'vpn_connect_failed') {
    const reason = String((payload.props && payload.props.reason) || 'unknown').slice(0, 120);
    failReasons.set(reason, (failReasons.get(reason) ?? 0) + 1);
  }
}

async function flushDigest() {
  if (eventCounts.size === 0) return;
  const lines = [
    `Wawity · сводка за ${digestMinutes} мин`,
    `Уникальных сессий сегодня: ${sessionsToday.size}`,
  ];
  for (const [key, count] of [...eventCounts.entries()].sort((a, b) => b[1] - a[1])) {
    lines.push(`${key}: ${count}`);
  }
  if (failReasons.size > 0) {
    lines.push('— причины неудачных подключений —');
    for (const [reason, count] of [...failReasons.entries()].sort((a, b) => b[1] - a[1]).slice(0, 8)) {
      lines.push(`${count}× ${reason}`);
    }
  }
  eventCounts = new Map();
  failReasons = new Map();
  await sendTelegram(lines.join('\n').slice(0, 3900));
}

setInterval(flushDigest, digestMinutes * 60000);

const ingestPath = `/e/${ingestKey}`;

http
  .createServer((req, res) => {
    const ip = String(req.headers['x-forwarded-for'] || req.socket.remoteAddress || '')
      .split(',')[0]
      .trim();
    if (req.method !== 'POST' || req.url !== ingestPath) {
      res.writeHead(404);
      res.end();
      return;
    }
    if (!allow(ip)) {
      res.writeHead(429);
      res.end();
      return;
    }
    let size = 0;
    const chunks = [];
    req.on('data', (chunk) => {
      size += chunk.length;
      if (size > 8192) req.destroy();
      else chunks.push(chunk);
    });
    req.on('end', () => {
      try {
        takeEvent(JSON.parse(Buffer.concat(chunks).toString('utf8')));
      } catch {}
      res.writeHead(204);
      res.end();
    });
  })
  .listen(port, () => {
    console.log(`wawity relay listening on :${port}`);
  });
