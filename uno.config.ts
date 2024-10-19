import { defineConfig, presetIcons, presetUno, transformerCompileClass } from 'unocss'

export default defineConfig({
  presets: [
    presetUno({variablePrefix: '_'}),
    presetIcons({
      collections: {
        ms: () => import('@iconify-json/material-symbols/icons.json').then(i => i.default as any)
      }
    })
  ],
  transformers: [transformerCompileClass({classPrefix: '_'})],
  rules: [
    [/^ga-(\w+)$/, (m) => ({'grid-area': m[1]})],
  ],
  variants: [
    m => m.startsWith('sel:') ? ({matcher: m.slice(4), selector: s => s + ' input:checked + span'}) : m
  ],
  shortcuts: [
    {f: 'flex', g: 'grid', 'stick-top': 'sticky top-0', 'stick-bottom': 'sticky bottom-0'},
    {'has-before': 'before:content-[""]', 'has-after': 'after:content-[""]'},
    [/^bgvar-(.+)$/, ([, c]) => `bg-[var(--${c})]`]
  ],
  theme: {
    colors: {
      w: '#fff',
    }
  }
})