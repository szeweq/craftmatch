import { wsModErrors } from '$lib/ws.js';

export async function load({ params }) {
    return {errors: (await wsModErrors(params.id as FileID)).map<[Date, string, string]>(([d, ...e]) => [new Date(1000 * d), ...e])}
}