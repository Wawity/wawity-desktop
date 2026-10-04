/**
 * Downscale a picked picture so it stays light in memory and in IndexedDB.
 * GIFs are kept as-is so animation survives.
 */
export async function fileToImageBlob(file: File, maxSide: number): Promise<Blob> {
  if (!file.type.startsWith('image/')) throw new Error('not-an-image');
  if (file.type === 'image/gif') {
    if (file.size > 12 * 1024 * 1024) throw new Error('too-big');
    return file;
  }
  const bmp = await createImageBitmap(file);
  const scale = Math.min(1, maxSide / Math.max(bmp.width, bmp.height));
  const w = Math.max(1, Math.round(bmp.width * scale));
  const h = Math.max(1, Math.round(bmp.height * scale));
  const canvas = document.createElement('canvas');
  canvas.width = w;
  canvas.height = h;
  const ctx = canvas.getContext('2d');
  if (!ctx) throw new Error('no-canvas');
  ctx.imageSmoothingQuality = 'high';
  ctx.drawImage(bmp, 0, 0, w, h);
  bmp.close();
  return new Promise<Blob>((resolve, reject) => {
    canvas.toBlob((b) => (b ? resolve(b) : reject(new Error('encode'))), 'image/webp', 0.9);
  });
}

/** Opens the system file picker and resolves with the chosen file (or null). */
export function pickImageFile(): Promise<File | null> {
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = 'image/png,image/jpeg,image/webp,image/gif,image/avif';
    input.onchange = () => resolve(input.files?.[0] ?? null);
    input.addEventListener('cancel', () => resolve(null));
    input.click();
  });
}
