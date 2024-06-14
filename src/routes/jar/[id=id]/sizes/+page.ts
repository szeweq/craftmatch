import { wsContentSizes } from "$lib/ws";

export async function load({ params }) {
    return {sizes: await wsContentSizes(params.id as FileID)}
}