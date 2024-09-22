import srv from "$lib/srv"

export const ssr = false
export const load = () => srv.sync().then(() => ({}))