export interface AccentPalette {
  id: string;
  ru: string;
  en: string;
  hex: string;
}

export const ACCENTS: AccentPalette[] = [
  { id: 'ember', ru: 'Уголь', en: 'Ember', hex: '#fe2c02' },
  { id: 'solar', ru: 'Вспышка', en: 'Solar', hex: '#ffb020' },
  { id: 'aurora', ru: 'Аврора', en: 'Aurora', hex: '#2ee59d' },
  { id: 'pulsar', ru: 'Пульсар', en: 'Pulsar', hex: '#3cc8ff' },
  { id: 'nebula', ru: 'Туманность', en: 'Nebula', hex: '#9b5cff' },
  { id: 'sakura', ru: 'Сакура', en: 'Sakura', hex: '#ff5c9a' },
  { id: 'blood', ru: 'Кровь', en: 'Blood', hex: '#e0103a' },
  { id: 'ice', ru: 'Лёд', en: 'Ice', hex: '#9fe7ff' },
  { id: 'mono', ru: 'Моно', en: 'Mono', hex: '#e9e6e2' },
];

export function accentHex(id: string, custom = '#fe2c02'): string {
  if (id === 'custom') return /^#[0-9a-f]{6}$/i.test(custom) ? custom : '#fe2c02';
  return ACCENTS.find((a) => a.id === id)?.hex ?? '#fe2c02';
}
