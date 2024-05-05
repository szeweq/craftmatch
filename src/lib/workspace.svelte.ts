import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import { wsFiles } from "./ws"

let wsOpen = $state(false)
let files = $state<[UUID, string][]>([])
let loadState = $state(0)

listen("ws-open", e => wsOpen = !!e.payload).then(() => invoke("load"))

export const ws = {
    get open() { return wsOpen },
    get files() { return files },
    loadFiles() {
        if (loadState > 0) return
        wsFiles(d => files = d)
        loadState = 1
    }
}