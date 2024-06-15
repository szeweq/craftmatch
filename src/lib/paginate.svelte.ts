
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