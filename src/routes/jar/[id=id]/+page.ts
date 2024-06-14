import { wsModData } from '$lib/ws'

export function load({ params }) {
    return wsModData(params.id as FileID)
}