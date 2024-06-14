import { wsStrIndex } from '$lib/ws'

export function load({ params }) {
    return wsStrIndex(params.id as FileID)
}