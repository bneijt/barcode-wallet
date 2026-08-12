# Barcode Wallet

A progressive web app that stores loyalty membership barcodes on your device and displays them full-screen at high contrast so they can be scanned at the point of sale. All data lives client-side only — there is no server, no account, and no cloud storage.

## Features

- **Overview** — a color-coded grid of tiles, one per Code. Each tile shows the Code's name, its ordinal among same-named Codes (1st, 2nd, 3rd…), and its symbology type (QR, EAN, Code 128…).
- **Full-screen display** — tap a tile to show the Code full-screen, high contrast and large, so it scans reliably at typical phone brightness. Return with the back button or a back swipe.
- **Add Codes** — via the **+** button, choose one of three input methods:
  - **Camera** — scan a barcode live (uses the rear camera).
  - **Image** — decode a barcode from a photo.
  - **Manual** — type the Code value by hand.
- **Edit** — change a Code's name and color, or delete it with confirmation. A Code's value and symbology are immutable once added.
- **Export / Import** — back up or move your whole collection as JSON (hamburger menu). Useful when switching browsers or moving between a browser and a home-screen app. Import validates each record individually: new Codes are added, duplicates are skipped, invalid records are rejected, and a summary is shown.
- **Privacy by design** — everything stays in your browser's IndexedDB. No logging, no analytics, no cloud storage. The service worker only caches the app's own files so it works offline.
- **PWA** — installable and works offline.

Supported symbologies so far: **Code 128, EAN-13, UPC-A, QR**.

## Installing

Barcode Wallet is a web app — there's nothing to download from an app store.

1. Open the app URL in your phone's browser: **https://barcode-wallet.bneijt.nl**
2. Use it directly in the browser, or add it to your home screen:
   - **iPhone (Safari):** tap **Share** → **Add to Home Screen**.
   - **Android (Chrome):** tap the **⋮** menu → **Add to Home screen** (or **Install app**).

Installing gives you a full-screen app experience and offline access.

## Using the app

### Add a Code

1. Tap the **+** button.
2. Choose an input method:
   - **Camera** — point at the barcode; it decodes automatically.
   - **Image** — pick a photo of the barcode.
   - **Manual** — select the symbology and type the value.
3. Give the Code a name and pick a color, then tap **Save**.

### Show a Code at the till

Tap a tile in the overview to show it full-screen, then hold your phone up to the scanner. Tap **‹** or swipe back to return.

### Edit or delete a Code

Tap a Code to display it, then tap the **✎** button. Change its name or color and save, or delete it (with confirmation).

### Back up or move your Codes

1. Tap the hamburger menu (**☰**).
2. **Export** downloads a `barcode-wallet-<date>.json` file of your whole collection.
3. On another device or browser, tap **Import** and select that file to restore it.

## Development

### Requirements

- [Rust](https://rustup.rs) with the `wasm32-unknown-unknown` target
- [Trunk](https://trunkrs.dev)

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk
```

### Run locally

```sh
./start.sh        # serves at http://localhost:8080
```

## License

This project is licensed under the GNU Affero General Public License v3.0.
