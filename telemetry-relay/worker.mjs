const buckets = new Map();

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
  if (buckets.size > 5000) buckets.clear();
  return true;
}

function escapeHtml(value) {
  return String(value)
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;');
}

function flagEmoji(code) {
  if (!/^[A-Z]{2}$/.test(code)) return '';
  return String.fromCodePoint(...[...code].map((ch) => ch.charCodeAt(0) + 127397));
}

function formatEvent(payload, geo) {
  const name = String(payload.event || '').slice(0, 64);
  if (!name) return null;
  const version = String(payload.appVersion || '?').slice(0, 32);
  const os = String(payload.os || '?').slice(0, 16);
  const props =
    payload.props && typeof payload.props === 'object' && !Array.isArray(payload.props)
      ? payload.props
      : {};
  const lines = [];
  if (name === 'app_started') {
    lines.push('🚀 <b>Запуск приложения</b>');
  } else if (name === 'vpn_connected') {
    lines.push('🟢 <b>Подключение к VPN</b>');
    const bits = [];
    if (props.protocol) bits.push('протокол: ' + escapeHtml(String(props.protocol).slice(0, 24)));
    if (props.multihop === true || props.multihop === 'true') bits.push('мультихоп');
    if (bits.length) lines.push(bits.join(' · '));
  } else if (name === 'vpn_connect_failed') {
    lines.push('🔴 <b>Не удалось подключиться</b>');
    lines.push('<code>' + escapeHtml(String(props.reason || 'unknown').slice(0, 300)) + '</code>');
  } else if (name === 'relay_test') {
    lines.push('🛠 <b>Тестовое событие</b>');
  } else {
    lines.push('📦 <b>' + escapeHtml(name) + '</b>');
    for (const [key, value] of Object.entries(props).slice(0, 6)) {
      lines.push(escapeHtml(String(key).slice(0, 40)) + ': ' + escapeHtml(String(value).slice(0, 120)));
    }
  }
  const footer = ['v' + escapeHtml(version), escapeHtml(os)];
  const country = String((geo && geo.country) || '').toUpperCase();
  if (country) {
    const flag = flagEmoji(country);
    const city = geo && geo.city ? ', ' + escapeHtml(String(geo.city).slice(0, 40)) : '';
    footer.push((flag ? flag + ' ' : '') + country + city);
  }
  const session = String(payload.sessionId || '').slice(-6);
  if (session) footer.push('#' + escapeHtml(session));
  lines.push('<i>' + footer.join(' · ') + '</i>');
  return lines.join('\n').slice(0, 3900);
}

export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);
    if (request.method !== 'POST' || url.pathname !== '/e/' + env.INGEST_KEY) {
      return new Response('not found', { status: 404 });
    }
    const ip = request.headers.get('cf-connecting-ip') || '';
    if (!allow(ip)) {
      return new Response(null, { status: 429 });
    }
    let payload;
    try {
      const raw = await request.text();
      if (raw.length > 8192) {
        return new Response(null, { status: 413 });
      }
      payload = JSON.parse(raw);
    } catch {
      return new Response('bad json', { status: 400 });
    }
    const text = formatEvent(payload, request.cf || {});
    if (text) {
      ctx.waitUntil(
        fetch('https://api.telegram.org/bot' + env.TG_BOT_TOKEN + '/sendMessage', {
          method: 'POST',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify({
            chat_id: env.TG_CHAT_ID,
            text,
            parse_mode: 'HTML',
            disable_notification: true,
          }),
        })
          .then(async (tg) => {
            if (!tg.ok) {
              console.error('telegram error', tg.status, await tg.text());
            }
          })
          .catch((err) => {
            console.error('telegram unreachable', String(err));
          }),
      );
    }
    return new Response(null, { status: 204 });
  },
};
