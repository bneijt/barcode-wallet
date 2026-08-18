// The app registers this worker as /sw.js?v=<version>. Deriving the cache name
// from the URL keeps this file static while rotating the cache on each release
// (old caches are pruned on activate, keeping hashed assets from accumulating).
const VERSION = new URLSearchParams(self.location.search).get("v") || "dev";
const CACHE = `barcode-wallet-${VERSION}`;
const ASSETS = [
  "/",
  "/manifest.webmanifest",
  "/favicon.svg",
  "/icon-192.png",
  "/icon-512.png",
];

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches.open(CACHE).then((cache) => cache.addAll(ASSETS)).then(() => self.skipWaiting())
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) =>
        Promise.all(keys.filter((k) => k !== CACHE).map((k) => caches.delete(k)))
      )
      .then(() => self.clients.claim())
  );
});

self.addEventListener("fetch", (event) => {
  if (event.request.method !== "GET") return;

  if (event.request.mode === "navigate") {
    // Network-first for the HTML shell so the app always reflects the latest
    // deploy; the referenced wasm/js/css use hashed filenames, so a fresh
    // index.html pulls the new assets. Fall back to cache only when offline.
    event.respondWith(
      fetch(event.request)
        .then((response) => {
          const copy = response.clone();
          caches.open(CACHE).then((cache) => cache.put(event.request, copy));
          return response;
        })
        .catch(() =>
          caches.match(event.request).then((cached) => cached || caches.match("/"))
        )
    );
    return;
  }

  // Cache-first for hashed assets and static files. If the network request
  // fails and nothing is cached, surface a controlled response instead of an
  // unhandled NetworkError rejection.
  event.respondWith(
    caches.match(event.request).then((cached) => {
      if (cached) return cached;
      return fetch(event.request)
        .then((response) => {
          const copy = response.clone();
          caches.open(CACHE).then((cache) => cache.put(event.request, copy));
          return response;
        })
        .catch(() => new Response("", { status: 503 }));
    })
  );
});
