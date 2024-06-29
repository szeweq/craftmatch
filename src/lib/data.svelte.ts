
type Source<S> = (() => S) | {readonly out: S}
const canon = <S>(s: Source<S>) => typeof s === "function" ? s : () => s.out

export function queryable<T>(source: Source<T[]>, by: (x: T) => string) {
  source = canon(source)
  let q = $state("")
  let out = $derived.by(() => {
    const t = source(), qq = q.toLowerCase()
    return qq ? t.filter(x => by(x).toLowerCase().includes(qq)) : t
  })
  return {
    get q() {return q},
    set q(v) {q = v},
    get all() {return source().length},
    get out() {return out},
    [Symbol.iterator]() {return out[Symbol.iterator]()}
  }
}
export type Queryable<T> = ReturnType<typeof queryable<T>>

const perPage = 40

export function paginate<T>(source: Source<T[]>) {
  source = canon(source)
  let page = $state(0)
  let pages = $derived(Math.ceil(source().length / perPage))
  let chunk = $derived(source().slice(page * perPage, (page + 1) * perPage))
  $effect(() => {
    if (page >= pages) page = pages - 1
  })
  return {
    get current() {return page},
    set current(v) {page = v},
    get all() {return pages},
    [Symbol.iterator]() {return chunk[Symbol.iterator]()}
  }
}
export type Paginate<T> = ReturnType<typeof paginate<T>>

export function sortable<T>(source: Source<T[]>, by: (x: T) => any) {
  source = canon(source)
  let sortID = $state(0)
  let out = $derived.by(() => {
    const t = source()
    if (sortID === 0) return t
    let s = t.map<[number, any]>((x, i) => [i, by(x)]).sort((a, b) => b[1] - a[1])
    if (sortID > 1) s.reverse()
    return s.map(x => t[x[0]])
  })
  return {
    get sortID() {return sortID},
    set sortID(v) {sortID = v},
    get out() {return out},
    [Symbol.iterator]() {return out[Symbol.iterator]()}
  }
}