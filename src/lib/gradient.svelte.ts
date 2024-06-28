
const imgData = (src: string) => new Promise<Uint8ClampedArray>((ok, err) => {
  const c = document.createElement("canvas"), ctx = c.getContext("2d"), img = new Image
  img.onload = () => {
    c.width = img.width
    c.height = img.height
    ctx.drawImage(img, 0, 0)
    ok(ctx.getImageData(0, 0, img.width, img.height).data)
  }
  img.onerror = () => err(new Error("Failed to load image"))
  img.crossOrigin = ''
  img.src = src
})
const hex = (n: number) => (n|0).toString(16).padStart(2, '0')
const grayscale = (r: number, g: number, b: number) => 0.299 * r + 0.587 * g + 0.114 * b
const rgb2hsl = (r: number, g: number, b: number) => {
  r /= 255, g /= 255, b /= 255
  const max = Math.max(r, g, b), min = Math.min(r, g, b), d = max - min
  let h = 0, s = 0, l = (max + min) / 2
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
  const q = l < 0.5 ? l * (1 + s) : l + s - l * s
  const p = 2 * l - q
  const [r, g, b] = [h + 1 / 3, h, h - 1 / 3].map(x => {
    if (x < 0) x += 1
    if (x > 1) x -= 1
    if (x < 1 / 6) return p + (q - p) * 6 * x
    if (x < 1 / 2) return q
    if (x < 2 / 3) return p + (q - p) * (2 / 3 - x) * 6
    return p
  })
  return [r * 255, g * 255, b * 255]
}
const relighten = (r: number, g: number, b: number, l: number) => {
  const [h, s, _] = rgb2hsl(r, g, b)
  return hsl2rgb(h, s, l)
}
const findColors = (d: Uint8ClampedArray) => {
  console.time("fc")
  const gap = 40, cols = new Map<string, number>
  for (let i = 0; i < d.length; i += gap) {
    let [r, g, b] = d.subarray(i, i + 3);
    const [h, s, l] = rgb2hsl(r, g, b)
    if (s < 0.2) continue
    [r, g, b] = relighten(r, g, b, Math.min(Math.max(grayscale(r, g, b), 48), 64) / 255)
    let c = '#' + hex(r) + hex(g) + hex(b)
    cols.set(c, (cols.get(c) ?? 0) + 1)
  }
  const z = [...cols.entries()].sort((a, b) => b[1] - a[1]).map(x => x[0]).slice(0, 4)
  console.timeEnd("fc")
  return z
}

export function mainColors(src: () => string) {
  let bg = $state("none")

  return {
    get bg() {return bg},
    compute() {
      const s = src()
      s && imgData(s).then(d => bg = findColors(d)?.[0] ?? 'none')
    }
  }
}
