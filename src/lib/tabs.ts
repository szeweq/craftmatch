import { type SvelteComponent, type ComponentType } from "svelte"

export type Tab<T extends Record<string, any>> = {
  name: string,
  component: ComponentType<SvelteComponent<T>>,
  content: T
}

type TabAdder = <T extends Record<string, any>>(name: string, component: Tab<T>['component'], content: T) => void
type TabRenamer = (key: UUID, name: string) => void

let tabAdder: TabAdder = () => {}
let tabRenamer: TabRenamer = () => {}

export function setAdder(add: TabAdder) {
  tabAdder = add
}
export function setRenamer(rename: TabRenamer) {
  tabRenamer = rename
}
export function addTab<T extends Record<string, any>>(name: string, component: Tab<T>['component'], content: T = {} as any) {
  tabAdder(name, component, content)
}
export function renameTab(id: UUID, name: string) {
  tabRenamer(id, name)
}
export function linkTab<T extends Record<string, any>>(name: string, component: Tab<T>['component'], content: T = {} as any) {
  return () => tabAdder(name, component, content)
}