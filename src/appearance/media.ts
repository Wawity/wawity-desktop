/**
 * Video picking. Unlike pictures there is no downscale pass: a clip is already
 * compressed, and re-encoding one in the browser would take longer than the
 * user would tolerate. The only guard is a size ceiling, because a 4K clip
 * would sit in IndexedDB forever and never decode.
 */
export const VIDEO_MAX_BYTES = 220 * 1024 * 1024;

export const VIDEO_EXT = ['mp4', 'webm', 'ogv', 'mov'];

/** mp4 first: it is the one every WebView2 plays without a codec question. */
export const VIDEO_MIME = 'video/mp4,video/webm,video/ogg,video/quicktime';

export function pickVideoFile(): Promise<File | null> {
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = VIDEO_MIME;
    input.onchange = () => resolve(input.files?.[0] ?? null);
    input.addEventListener('cancel', () => resolve(null));
    input.click();
  });
}

export function checkVideo(file: File): string | null {
  if (!file.type.startsWith('video/')) return 'not-a-video';
  if (file.size > VIDEO_MAX_BYTES) return 'too-big';
  return null;
}