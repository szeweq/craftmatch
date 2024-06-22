import { invoke } from "@tauri-apps/api/core"
import { listen, once } from "@tauri-apps/api/event"

let name = $state("")
let avatar = $state("")
let status = $state(0)

listen<[string, string, number] | null>("auth", e => {
  let [u, a, s] = e.payload ?? ["", "", 0]
  name = u
  avatar = a
  status = s
})

export async function doAuth(passCode: (code: string, url: string) => void) {
  const p = invoke<boolean>("auth")
  once<[string, string]>("authcode", e => passCode(...e.payload))
  return await p
}

export const user = {
  get name() { return name },
  get avatar() { return avatar },
  get status() { return status }
}