import { invoke } from "@tauri-apps/api/core"

let port = 9267

export default {
  get port() { return port },
  sync() { return invoke<number>("srv_port").then(p => port = p) },
  url(path: string) { return `http://localhost:${port}${path}` }
}