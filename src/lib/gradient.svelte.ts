const htmlImg = (img: HTMLImageElement) => new Promise<Uint8ClampedArray>(ok => {
  const fn = () => ok(drawImage(img))
  if (img.complete) fn(); else img.onload = fn
})
const drawImage = (img: HTMLImageElement) => {
  const c = document.createElement("canvas"), ctx = c.getContext("2d")
  c.width = img.width
  c.height = img.height
  ctx.drawImage(img, 0, 0)
  return ctx.getImageData(0, 0, img.width, img.height).data
}
const hex = (n: number) => (n|0).toString(16).padStart(2, '0')
const rgb2hsl = (r: number, g: number, b: number) => {
  r /= 255, g /= 255, b /= 255
  const max = Math.max(r, g, b), min = Math.min(r, g, b), d = max - min, l = (max + min) / 2
  let h = 0, s = 0
  if (max !== min) {
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min)
    switch (max) {
      case r: h = (g - b) / d + (g < b ? 6 : 0); break
      case g: h = (b - r) / d + 2; break
      case b: h = (r - g) / d + 4; break
    }
    h /= 6
  }
  return [h, s, l]
}
const hsl2rgb = (h: number, s: number, l: number) => {
  const q = l < 0.5 ? l * (1 + s) : l + s - l * s, p = 2 * l - q
  return [h + 1 / 3, h, h - 1 / 3].map(x => {
    if (x < 0) x += 1
    if (x > 1) x -= 1
    if (x < 1 / 6) return p + (q - p) * 6 * x
    if (x < 1 / 2) return q
    if (x < 2 / 3) return p + (q - p) * (2 / 3 - x) * 6
    return p
  }).map(x => x * 255)
}
const findColor = (d: Uint8ClampedArray) => {
  const gap = 40, cols = new Map<string, number>
  for (let i = 0; i < d.length; i += gap) {
    let [r, g, b] = d.subarray(i, i + 3);
    const [h, s, l] = rgb2hsl(r, g, b)
    if (s < 0.2) continue
    [r, g, b] = hsl2rgb(h, s, Math.min(Math.max(l, 0.1875), 0.25))
    const c = hex(r) + hex(g) + hex(b)

    cols.set(c, (cols.get(c) ?? 0) + 1)
  }
  let col = "", cnum = 0
  for (const [c, n] of cols) {
    if (n > cnum) {
      col = c
      cnum = n
    }
  }
  return cnum ? '#' + col : 'none'
}

export function imgColors(src: () => HTMLImageElement) {
  let bg = $state("none")

  return {
    get bg() {return bg},
    compute() {
      const s = src()
      s && htmlImg(s).then(d => bg = findColor(d))
    }
  }
}