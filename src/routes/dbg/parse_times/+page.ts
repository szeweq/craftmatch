import { invoke } from "@tauri-apps/api/core"

export async function load() {
    return await invoke<Record<string, number>>('dbg_parse_times')
}