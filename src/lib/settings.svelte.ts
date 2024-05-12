
let loc = $state(!!localStorage.getItem("loc"))
let lang = $derived(loc ? navigator.language : "en")

export const settings = {
    get loc() { return loc },
    set loc(v) {
        loc = v
        localStorage.setItem("loc", ''+v)
    },
    get lang() { return lang }
}