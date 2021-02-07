"use-strict";

addEventListener('fetch', event => {
    event.respondWith(handleRequest(event.request))
})

/**
 * Fetch and log a request
 * @param {Request} request
 */
async function handleRequest(request) {
    // Assert env vars present
    console.assert(client_secret !== undefined);
    console.assert(client_id !== undefined);
    console.assert(redirect_url !== undefined);

    /** @type {import("../pkg/twilight_test")} */
    const { handle_request } = wasm_bindgen;

    await wasm_bindgen(wasm);

    return handle_request(request);
}
