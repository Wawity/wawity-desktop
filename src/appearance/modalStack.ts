const stack: number[] = [];
let serial = 0;
let layer = 1200;

export function claimModalId() {
  return ++serial;
}

export function pushModal(id: number) {
  if (!stack.includes(id)) stack.push(id);
  document.documentElement.classList.add('wawity-look-open');
  return ++layer;
}

export function isTopModal(id: number) {
  return stack[stack.length - 1] === id;
}

export function popModal(id: number) {
  const index = stack.indexOf(id);
  if (index !== -1) stack.splice(index, 1);
  if (!stack.length) {
    document.documentElement.classList.remove('wawity-look-open');
    layer = 1200;
  }
}
