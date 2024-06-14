import { wsFileTypeSizes } from '$lib/ws.js';

export async function load({ params }) {
    return {sizes: await wsFileTypeSizes(params.id as FileID)}
}