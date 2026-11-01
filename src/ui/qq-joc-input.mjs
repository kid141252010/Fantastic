// Adapt unencrypted QQ EC-3 files to the same fMP4 input used by Apple's JOC
// Worker. This module only reads containers; decoding and rendering stay there.
const RATE = 48000;
const MAX_SAMPLES = 1_000_000;
const DOWNLOAD = /^\/qq\/song\/[0-9A-Za-z]{1,32}\/download\?quality=DAEAC3$/;
const fail = (message) => { throw new Error(`QQ JOC: ${message}`); };
const aborted = (signal) => { if (signal?.aborted) throw signal.reason || new DOMException('Aborted', 'AbortError'); };

function reader(bytes) {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const u32 = (pos) => { if (pos < 0 || pos + 4 > bytes.length) fail('truncated MP4 table'); return view.getUint32(pos); };
  const u64 = (pos) => { const value = u32(pos) * 0x100000000 + u32(pos + 4); if (!Number.isSafeInteger(value)) fail('oversized MP4 offset'); return value; };
  function boxes(start = 0, end = bytes.length) {
    const out = [];
    for (let pos = start; pos < end;) {
      if (pos + 8 > end) fail('truncated MP4 box');
      let size = u32(pos), header = 8;
      const type = String.fromCharCode(...bytes.subarray(pos + 4, pos + 8));
      if (size === 1) { size = u64(pos + 8); header = 16; }
      else if (size === 0) size = end - pos;
      if (size < header || size > end - pos) fail(`invalid ${type} box`);
      out.push({ type, start: pos, body: pos + header, end: pos + size });
      pos += size;
    }
    return out;
  }
  const children = (parent, skip = 0) => boxes(parent.body + skip, parent.end);
  const child = (parent, type) => children(parent).find((box) => box.type === type) || fail(`missing ${type}`);
  function table(box, width, skip = 8) {
    const count = u32(box.body + 4);
    if (count > MAX_SAMPLES || box.body + skip + count * width > box.end) fail(`invalid ${box.type} table`);
    return Array.from({ length: count }, (_, i) => box.body + skip + i * width);
  }
  return { u32, u64, boxes, children, child, table, signed: (pos) => view.getInt32(pos) };
}

// A sample may contain the independent syncframe followed by dependent streams.
function syncframes(bytes) {
  const frames = [];
  for (let pos = 0; pos < bytes.length;) {
    if (pos + 6 > bytes.length || bytes[pos] !== 0x0b || bytes[pos + 1] !== 0x77 || (bytes[pos + 5] >> 3) <= 10) fail('invalid E-AC-3 syncframe');
    const size = ((((bytes[pos + 2] & 7) << 8) | bytes[pos + 3]) + 1) * 2;
    if (size < 6 || pos + size > bytes.length) fail('truncated E-AC-3 syncframe');
    const fscod = bytes[pos + 4] >> 6;
    if (fscod !== 0) fail('JOC rendering requires 48 kHz E-AC-3');
    const type = bytes[pos + 2] >> 6, substream = (bytes[pos + 2] >> 3) & 7;
    if (type === 3 || (type !== 1 && substream !== 0)) fail('unsupported E-AC-3 substream');
    frames.push({ bytes: bytes.subarray(pos, pos + size), dependent: type === 1,
      samples: [1, 2, 3, 6][(bytes[pos + 4] >> 4) & 3] * 256 });
    pos += size;
  }
  if (!frames.length || frames[0].dependent) fail('missing independent E-AC-3 syncframe');
  return frames;
}

function rawSamples(bytes) {
  const samples = [];
  let current = null;
  for (const frame of syncframes(bytes)) {
    if (!frame.dependent) { current = { parts: [], duration: frame.samples / RATE }; samples.push(current); }
    current.parts.push(frame.bytes);
  }
  return samples;
}

function mp4Samples(bytes) {
  const r = reader(bytes), top = r.boxes();
  const moov = top.find((box) => box.type === 'moov') || fail('missing moov');
  const mdats = top.filter((box) => box.type === 'mdat');
  let track = null;
  for (const trak of r.children(moov).filter((box) => box.type === 'trak')) {
    const mdia = r.child(trak, 'mdia'), stbl = r.child(r.child(mdia, 'minf'), 'stbl');
    const entries = r.children(r.child(stbl, 'stsd'), 8);
    const description = entries.findIndex((box) => box.type === 'ec-3') + 1;
    if (!description) continue;
    if (track) fail('multiple E-AC-3 tracks');
    const tkhd = r.child(trak, 'tkhd'), mdhd = r.child(mdia, 'mdhd');
    const id = r.u32(tkhd.body + (bytes[tkhd.body] === 1 ? 20 : 12));
    const scale = r.u32(mdhd.body + (bytes[mdhd.body] === 1 ? 20 : 12));
    if (!scale) fail('invalid MP4 timescale');
    track = { id, scale, stbl, description };
  }
  if (!track) fail('MP4 has no unencrypted ec-3 track');
  const samples = [];
  const add = (offset, size, duration) => {
    if (!size || !mdats.some((mdat) => offset >= mdat.body && size <= mdat.end - offset)) fail('invalid MP4 sample range');
    const data = bytes.subarray(offset, offset + size), frames = syncframes(data);
    const seconds = duration ? duration / track.scale : frames.filter((f) => !f.dependent).reduce((n, f) => n + f.samples / RATE, 0);
    if (!(seconds > 0) || samples.length >= MAX_SAMPLES) fail('invalid MP4 sample duration');
    samples.push({ parts: [data], duration: seconds });
  };
  const fragments = top.filter((box) => box.type === 'moof');
  if (fragments.length) {
    const mvex = r.children(moov).find((box) => box.type === 'mvex');
    const trex = mvex && r.children(mvex).find((box) => box.type === 'trex' && r.u32(box.body + 4) === track.id);
    const defaults = trex ? { description: r.u32(trex.body + 8), duration: r.u32(trex.body + 12), size: r.u32(trex.body + 16) } : {};
    for (const moof of fragments) {
      const trafs = r.children(moof).filter((box) => box.type === 'traf');
      for (const traf of trafs) {
        const tfhd = r.child(traf, 'tfhd');
        if (r.u32(tfhd.body + 4) !== track.id) continue;
        const flags = r.u32(tfhd.body) & 0xffffff;
        if (trafs.length > 1 && !(flags & 1) && !(flags & 0x020000)) fail('ambiguous fragmented MP4 base offset');
        let pos = tfhd.body + 8;
        const base = flags & 1 ? r.u64(pos) : moof.start;
        if (flags & 1) pos += 8;
        const description = flags & 2 ? r.u32(pos) : defaults.description || 1;
        if (flags & 2) pos += 4;
        const duration = flags & 8 ? r.u32(pos) : defaults.duration;
        if (flags & 8) pos += 4;
        const size = flags & 16 ? r.u32(pos) : defaults.size;
        if (description !== track.description) fail('non-EC-3 sample description');
        let next = top.find((box) => box.type === 'mdat' && box.start >= moof.end)?.body;
        for (const trun of r.children(traf).filter((box) => box.type === 'trun')) {
          const runFlags = r.u32(trun.body) & 0xffffff, count = r.u32(trun.body + 4);
          let cursor = trun.body + 8;
          let offset = next;
          if (runFlags & 1) { offset = base + r.signed(cursor); cursor += 4; }
          else if (trafs.length > 1) fail('ambiguous fragmented MP4 data offset');
          if (runFlags & 4) cursor += 4;
          const fields = [0x100, 0x200, 0x400, 0x800].filter((flag) => runFlags & flag).length;
          if (count > MAX_SAMPLES || cursor + count * fields * 4 > trun.end) fail('invalid trun');
          for (let i = 0; i < count; i++) {
            const sampleDuration = runFlags & 0x100 ? r.u32(cursor) : duration;
            if (runFlags & 0x100) cursor += 4;
            const sampleSize = runFlags & 0x200 ? r.u32(cursor) : size;
            if (runFlags & 0x200) cursor += 4;
            if (runFlags & 0x400) cursor += 4;
            if (runFlags & 0x800) cursor += 4;
            add(offset, sampleSize, sampleDuration); offset += sampleSize;
          }
          next = offset;
        }
      }
    }
  } else {
    const boxes = r.children(track.stbl), get = (type) => boxes.find((box) => box.type === type) || fail(`missing ${type}`);
    const stsz = get('stsz'), fixed = r.u32(stsz.body + 4), count = r.u32(stsz.body + 8);
    if (!count || count > MAX_SAMPLES || (!fixed && stsz.body + 12 + count * 4 > stsz.end)) fail('invalid stsz');
    const sizes = Array.from({ length: count }, (_, i) => fixed || r.u32(stsz.body + 12 + i * 4));
    const stts = get('stts'), durations = [];
    for (const row of r.table(stts, 8)) {
      const n = r.u32(row), duration = r.u32(row + 4);
      if (!duration || n > count - durations.length) fail('invalid stts');
      for (let i = 0; i < n; i++) durations.push(duration);
    }
    if (durations.length !== count) fail('incomplete stts');
    const stsc = get('stsc'), mapping = r.table(stsc, 12).map((row) => ({ first: r.u32(row), count: r.u32(row + 4), description: r.u32(row + 8) }));
    if (!mapping.length || mapping[0].first !== 1 || mapping.some((row, i) => !row.count || row.description !== track.description || (i && row.first <= mapping[i - 1].first))) fail('invalid stsc');
    const offsets = boxes.find((box) => box.type === 'stco' || box.type === 'co64') || fail('missing chunk offsets');
    const chunks = r.table(offsets, offsets.type === 'co64' ? 8 : 4);
    let index = 0, map = 0;
    for (let chunk = 0; chunk < chunks.length; chunk++) {
      while (map + 1 < mapping.length && mapping[map + 1].first <= chunk + 1) map++;
      let offset = offsets.type === 'co64' ? r.u64(chunks[chunk]) : r.u32(chunks[chunk]);
      if (mapping[map].count > count - index) fail('too many chunk samples');
      for (let i = 0; i < mapping[map].count; i++, index++) { add(offset, sizes[index], durations[index]); offset += sizes[index]; }
    }
    if (index !== count) fail('incomplete chunk samples');
  }
  if (!samples.length) fail('empty E-AC-3 track');
  return samples;
}

const concat = (parts) => {
  const bytes = new Uint8Array(parts.reduce((n, part) => n + part.length, 0));
  let pos = 0; for (const part of parts) { bytes.set(part, pos); pos += part.length; } return bytes;
};
const words = (...values) => { const bytes = new Uint8Array(values.length * 4), view = new DataView(bytes.buffer); values.forEach((v, i) => view.setUint32(i * 4, v)); return bytes; };
const box = (type, ...parts) => concat([words(8 + parts.reduce((n, p) => n + p.length, 0)), Uint8Array.from(type, (c) => c.charCodeAt(0)), ...parts]);

function fragment(samples) {
  const packets = samples.map((sample) => concat(sample.parts));
  const makeMoof = (offset) => box('moof', box('traf', box('tfhd', words(0x020000, 1)),
    box('trun', words(0x201, packets.length, offset, ...packets.map((packet) => packet.length)))));
  const moof = makeMoof(0);
  return concat([makeMoof(moof.length + 8), box('mdat', ...packets)]).buffer;
}

/** Parse a downloaded MP4 or raw E-AC-3 file without touching the decoder. */
export function qqJocTrack(input) {
  const bytes = input instanceof Uint8Array ? input : new Uint8Array(input);
  const samples = bytes[0] === 0x0b && bytes[1] === 0x77 ? rawSamples(bytes) : mp4Samples(bytes);
  const segments = [];
  let duration = 0, start = 0, group = [];
  const flush = () => { if (group.length) { segments.push({ time: start, duration: duration - start, samples: group }); group = []; start = duration; } };
  for (const sample of samples) { group.push(sample); duration += sample.duration; if (duration - start >= 12) flush(); }
  flush();
  return { duration, segments, async load(segment, signal) {
    aborted(signal);
    if (!segments.includes(segment)) fail('invalid segment');
    // A new buffer every time: Worker transfer and seek must preserve the source.
    return fragment(segment.samples);
  } };
}

/** Only the backend may follow QQ's CDN URL or choose the regional egress. */
export async function openQqJocTrack(url, signal, onProgress) {
  if (typeof url !== 'string' || !DOWNLOAD.test(url)) fail('expected local QQ download URL');
  aborted(signal);
  const response = await fetch(url, { signal, credentials: 'same-origin', redirect: 'error' });
  if (!response.ok) fail(`download HTTP ${response.status}`);
  const lengthHeader = response.headers?.get?.('content-length') ?? response.headers?.['content-length'];
  const total = Number.isSafeInteger(Number(lengthHeader)) && Number(lengthHeader) > 0 ? Number(lengthHeader) : null;
  const report = (downloaded) => onProgress?.({ downloaded, total, percent: total ? Math.min(100, Math.floor(downloaded * 100 / total)) : null });
  let bytes;
  if (response.body?.getReader) {
    const stream = response.body.getReader();
    const chunks = [];
    let downloaded = 0;
    for (;;) {
      aborted(signal);
      const part = await stream.read();
      if (part.done) break;
      if (part.value?.length) {
        chunks.push(part.value);
        downloaded += part.value.length;
        report(downloaded);
      }
    }
    bytes = new Uint8Array(downloaded);
    let offset = 0;
    for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.length; }
  } else {
    const buffer = await response.arrayBuffer();
    bytes = new Uint8Array(buffer);
    report(bytes.length);
  }
  aborted(signal);
  return qqJocTrack(bytes);
}
