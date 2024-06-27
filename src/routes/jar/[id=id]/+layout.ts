import { invokeWS } from "$lib/ws"

export async function load({ params }) {
  const id = params.id as FileID
  const name = await invokeWS('ws_name', {id})
  return {id, name};
}