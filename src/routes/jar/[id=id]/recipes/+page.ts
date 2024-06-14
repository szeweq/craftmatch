import { wsRecipes } from "$lib/ws";

export async  function load({ params }) {
    return {recipes: await wsRecipes(params.id as FileID)}
}