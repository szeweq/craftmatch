import { wsDepMap } from '$lib/ws.js'

export async function load() {
  const [names, info] = await wsDepMap(false)
  return {names, info}
}