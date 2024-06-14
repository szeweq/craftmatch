
export function filterBy<T>(t: T[], q: string, by: (x: T) => string) {
    if (q === "") return t
    const qq = q.toLowerCase()
    return t.filter(x => by(x).toLowerCase().includes(qq))
}

export function sortBy<T>(t: T[], by?: ((x: T) => any) | false) {
    return !by ? t : t.map((x, i) => [i, by(x)]).sort((a, b) => b[1] - a[1]).map(x => t[x[0]])
}