import { wsFileTypeSizes } from '$lib/ws.js'

export async function load() {
    return {sizes: await wsFileTypeSizes(false)}
}