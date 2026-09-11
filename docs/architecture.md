# Leptos dashboard boundary

`eal-leptos-web` is the signed-in customer-facing read and interest-preview surface.

It does not query PostgreSQL directly, accept arbitrary crawl URLs, create match candidates during previews, open a process-local WebSocket, or send provider traffic. All index and match state comes from `eal-api` with server-side tenant context.

External indexes may increase discovery recall, but every URL must still pass the registered source policy and local fetch/extraction/embedding pipeline before appearing here.
