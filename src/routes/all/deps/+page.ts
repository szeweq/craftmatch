import { wsDepMap } from '$lib/ws.js'

export async function load() {
  const o = (await wsDepMap(false)).map<[string, {v: string, deps: Record<string, [string, string]>}]>(([n, v, deps]) => [n, {v, deps}])
  return Object.fromEntries(o)
}