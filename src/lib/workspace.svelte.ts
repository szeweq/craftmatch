import { invoke } from "@tauri-apps/api/core"
import { emit, listen } from "@tauri-apps/api/event"

type Files = [FileID, string, number][]
function wsFiles(f: (n: Files) => void) {
  invoke<Files>('ws_files', {force: false}).then(f)
  let p = listen<Files>('ws-files', e => f(e.payload))
  return () => p.then(f => f())
}

let wsOpen = $state(false)
let files = $state<Files>([])
let loadState = $state(0)

listen("ws-open", e => {
  const o = wsOpen
  wsOpen = !!e.payload
  if (o != wsOpen) loadState = 0
  if (!wsOpen) files = []
}).then(() => emit("load"))

export const ws = {
  get isOpen() { return wsOpen },
  get files() { return files },
  loadFiles() {
    wsOpen;
    if (loadState > 0) return
    wsFiles(d => files = d)
    loadState = 1
  },
  open() { return invoke("workspace", {open: true}) },
  close() { return invoke("workspace", {open: false}) },
}

export const dirs = <T>(d: string = null) => invoke<T>("dirs", {kind: d})