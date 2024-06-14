import { wsName } from "$lib/ws";

export async function load({ params }) {
    const name = await wsName(params.id as FileID)
    return {
        id: params.id as FileID,
        name
    };
}