
export function queryable<T>(source: () => T[], by: (x: T) => string) {
  let q = $state("")
  let queried = $derived.by(() => {
    const t = source(), qq = q.toLowerCase()
    if (qq === "") return t
    return t.filter(x => by(x).toLowerCase().includes(qq))
  })
  return {
    get q() {return q},
    set q(v) {q = v},
    get source() {return source()},
    get queried() {return queried},
  }
}
export type Queryable<T> = ReturnType<typeof queryable<T>>

const perPage = 40

export function paginate<T>(source: () => T[]) {
  let page = $state(0)
  let pages = $derived(Math.ceil(source().length / perPage))
  $effect(() => {
    if (page >= pages) page = pages - 1
  })
  return {
    get current() {return page},
    set current(v) {page = v},
    get all() {return pages},
    get chunk() {return source().slice(page * perPage, (page + 1) * perPage)}
  }
}
export type Paginate<T> = ReturnType<typeof paginate<T>>