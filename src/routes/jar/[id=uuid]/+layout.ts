import { wsName } from "$lib/ws";

export async function load({ params }) {
    const name = await wsName(params.id as UUID)
    return {
        id: params.id as UUID,
        name
    };
}