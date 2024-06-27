import { invoke } from "@tauri-apps/api/core"

type InvokeAPI = {
  workspace: [{open: boolean}, undefined],
  ws_show: [{id: FileID}, boolean],
  ws_name: [{id: FileID}, string],
  ws_mod_data: [{id: FileID}, ModData | null],
  ws_mod_playable: [{id: FileID}, string[] | null],
  ws_str_index: [{id: FileID}, StrIndex | null],
  ws_mod_entries: [{id: FileID}, {}],
}

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

export function invokeWS(cmd: keyof InvokeAPI, args?: InvokeAPI[typeof cmd][0]) {
  return invoke<InvokeAPI[typeof cmd][1]>(cmd, args)
}

function invokeWithMode<T>(cmd: string) {
  return (forceOrId: boolean | FileID) => invoke<T | null>(cmd, {
    mode: forceOrId
  })
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
