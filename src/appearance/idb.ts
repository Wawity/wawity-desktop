// Tiny IndexedDB wrapper for user pictures and sounds. localStorage is ~5 MB
// for the whole app (subscriptions live there too), so media must not go there.
const DB = 'wawity-appearance';
const IMAGES = 'images';
const AUDIO = 'audio';
const VIDEO = 'video';

let dbPromise: Promise<IDBDatabase> | null = null;

function open(): Promise<IDBDatabase> {
  if (dbPromise) return dbPromise;
  dbPromise = new Promise((resolve, reject) => {
    const req = indexedDB.open(DB, 3);
    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains(IMAGES)) db.createObjectStore(IMAGES);
      if (!db.objectStoreNames.contains(AUDIO)) db.createObjectStore(AUDIO);
      if (!db.objectStoreNames.contains(VIDEO)) db.createObjectStore(VIDEO);
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => {
      dbPromise = null;
      reject(req.error);
    };
  });
  return dbPromise;
}

async function run<T>(
  store: string,
  mode: IDBTransactionMode,
  fn: (s: IDBObjectStore) => IDBRequest<T>,
): Promise<T> {
  const db = await open();
  return new Promise<T>((resolve, reject) => {
    const tx = db.transaction(store, mode);
    const req = fn(tx.objectStore(store));
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

export async function putImage(key: string, blob: Blob): Promise<void> {
  await run(IMAGES, 'readwrite', (s) => s.put(blob, key));
}

export async function getImage(key: string): Promise<Blob | null> {
  const v = await run<unknown>(IMAGES, 'readonly', (s) => s.get(key));
  return v instanceof Blob ? v : null;
}

export async function delImage(key: string): Promise<void> {
  await run(IMAGES, 'readwrite', (s) => s.delete(key));
}

export async function putAudio(key: string, blob: Blob): Promise<void> {
  await run(AUDIO, 'readwrite', (s) => s.put(blob, key));
}

export async function getAudio(key: string): Promise<Blob | null> {
  const v = await run<unknown>(AUDIO, 'readonly', (s) => s.get(key));
  return v instanceof Blob ? v : null;
}

export async function delAudio(key: string): Promise<void> {
  await run(AUDIO, 'readwrite', (s) => s.delete(key));
}

/** Videos are stored raw. Re-encoding a clip in the browser costs tens of
 *  seconds and most of what a user picks is already an mp4/webm the WebView
 *  plays without help. */
export async function putVideo(key: string, blob: Blob): Promise<void> {
  await run(VIDEO, 'readwrite', (s) => s.put(blob, key));
}

export async function getVideo(key: string): Promise<Blob | null> {
  const v = await run<unknown>(VIDEO, 'readonly', (s) => s.get(key));
  return v instanceof Blob ? v : null;
}

export async function delVideo(key: string): Promise<void> {
  await run(VIDEO, 'readwrite', (s) => s.delete(key));
}
