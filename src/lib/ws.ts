import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"

export type Inheritance = {
  indices: [string, number][],
  inherits: number[][]
}
export type ClassCounting = {total: number, fields: number, methods: number, code: [string, number][]}
export type Complexity = {[k: string]: ClassCounting}
export type Tags = {[k: string]: {[k2: string]: {[k3: string]: number}}}
export type ModData = {
  type: "forge" | "fabric",
  mods: {
    name: string,
    slug: string,
    version: string,
    description?: string,
    authors?: string,
    license?: string,
    logo_path?: string
  }[]
}
export type StrIndex = {
  classes: string[],
  strings: [string, number[]][]
}
export type ContentTypes = 'meta' | 'classes' | 'assets' | 'data' | 'other'

function invokeWithMode<T>(cmd: string) {
  return (forceOrId: boolean | FileID) => invoke<T | null>(cmd, {
    mode: forceOrId
  })
}

export async function openWorkspace() {
  return await invoke<boolean>('open_workspace')
}

export async function wsShow(id: FileID) {
  return await invoke<boolean>('ws_show', {id})
}
export async function wsName(id: FileID) {
  return await invoke<string>('ws_name', {id})
}
export async function wsModData(id: FileID) {
  return await invoke<(ModData | null)>('ws_mod_data', {id})
}
export async function wsModPlayable(id: FileID) {
  return await invoke<(string[] | null)>('ws_mod_playable', {id})
}
export async function wsStrIndex(id: FileID) {
  return await invoke<(StrIndex | null)>('ws_str_index', {id})
}
export async function wsModEntries(id: FileID) {
  return await invoke<{}>('ws_mod_entries', {id})
}
export const wsFileTypeSizes = invokeWithMode<Record<string, [number, number, number]>>('ws_file_type_sizes')
export const wsContentSizes = invokeWithMode<Record<ContentTypes, [number, number, number]>>('ws_content_sizes')
export const wsInheritance = invokeWithMode<Inheritance>('ws_inheritance')
export const wsComplexity = invokeWithMode<Complexity>('ws_complexity')
export const wsTags = invokeWithMode<Tags>('ws_tags')
export const wsRecipes = invokeWithMode<Record<string, string[]>>('ws_recipes')


function invokeAndListen<T>(cmd: string, event: string) {
  return (f: (n: T) => void) => {
    invoke<T>(cmd, {force: false}).then(f)
    let p = listen<T>(event, e => f(e.payload))
    return () => p.then(f => f())
  }
}

export const wsFiles = invokeAndListen<[FileID, string, number][]>('ws_files', 'ws-files')
