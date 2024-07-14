import srv from "$lib/srv"

export const ssr = false

export async function load() {
  await srv.sync()
  return {}
}