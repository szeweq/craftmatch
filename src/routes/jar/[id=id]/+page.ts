import { wsModData, wsDepMap } from '$lib/ws'

export async function load({ params }) {
    const [md, [depNames, deps]] = await Promise.all([wsModData(params.id as FileID), wsDepMap(params.id as FileID)])
    return { ...md, depNames, deps }
}