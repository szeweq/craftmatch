import { filterBy } from "./query"

export function queryable<T>(source: () => T[], by: (x: T) => string) {
    let q = $state("")
    let queried = $derived(filterBy(source(), q, by))
    return {
        get q() {return q},
        set q(v) {q = v},
        get source() {return source()},
        get queried() {return queried},
    }
}

export type Queryable<T> = ReturnType<typeof queryable<T>>