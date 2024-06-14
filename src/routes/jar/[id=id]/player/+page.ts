import { wsModPlayable } from "$lib/ws";

export async function load({ params }) {
  return {files: await wsModPlayable(params.id as UUID)}
}