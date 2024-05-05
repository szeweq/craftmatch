const UUID_V7_REGEX = /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i

/** @type {import('@sveltejs/kit').ParamMatcher} */
export function match(param) {
  return UUID_V7_REGEX.test(param)
}