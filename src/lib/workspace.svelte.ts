import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import { wsFiles } from "./ws"

let wsOpen = $state(false)
let files = $state<[UUID, string, number][]>([])
let loadState = $state(0)

listen("ws-open", e => {
  const o = wsOpen
  wsOpen = !!e.payload
  if (o != wsOpen) loadState = 0
  if (!wsOpen) files = []
}).then(() => invoke("load"))

export const ws = {
  get open() { return wsOpen },
  get files() { return files },
  loadFiles() {
    wsOpen;
    if (loadState > 0) return
    wsFiles(d => files = d)
    loadState = 1
    }
}