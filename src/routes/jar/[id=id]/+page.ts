import { wsModData, wsDepMap } from '$lib/ws'

export async function load({ params }) {
    const [md, d] = await Promise.all([wsModData(params.id as FileID), wsDepMap(params.id as FileID)])
    const deps = Object.fromEntries(d.map<[string, {v: string, deps: Record<string, [string, string]>}]>(([n, v, deps]) => [n, {v, deps}]))
    return { deps, ...md }
}