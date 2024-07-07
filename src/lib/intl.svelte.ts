import { settings } from "./settings.svelte"

let langFmt = $derived(new Intl.DisplayNames(settings.lang, {type: 'language'}))
let _currentLang = $derived(langFmt.of(settings.lang))
let _dateFmt = $derived(new Intl.DateTimeFormat(settings.lang))
let _dateTimeFmt = $derived(new Intl.DateTimeFormat(settings.lang, {dateStyle: 'short', timeStyle: 'medium'}))
let _percentFmt = $derived(new Intl.NumberFormat(settings.lang, {style: 'percent', maximumFractionDigits: 2}))

export function useUnitFmt(unit: string, frac: number = 2) {
    let fmt = $derived(new Intl.NumberFormat(settings.lang, {style: 'unit', maximumFractionDigits: frac, unit, unitDisplay: 'short'}))
    return (n: number) => fmt.format(n)
}

export const currentLang = () => _currentLang
export const dateFmt = (d: number | Date) => _dateFmt.format(d)
export const dateTimeFmt = (d: number | Date) => _dateTimeFmt.format(d)
export const percentFmt = (n: number) => _percentFmt.format(n)
