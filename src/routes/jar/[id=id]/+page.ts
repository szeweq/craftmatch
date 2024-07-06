import { wsModData, wsDepMap } from '$lib/ws'

export async function load({ params }) {
    const [md, d] = await Promise.all([wsModData(params.id as FileID), wsDepMap(params.id as FileID)])
    const deps = Object.fromEntries(d)
    return { deps, ...md }
}