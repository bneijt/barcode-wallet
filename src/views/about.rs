use crate::AppState;
use crate::Screen;
use leptos::prelude::*;

const KOFI_URL: &str = "https://ko-fi.com/bneijt";

#[component]
pub fn About() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState");
    let back = move |_| state.screen.set(Screen::Overview);
    let version = env!("CARGO_PKG_VERSION");

    view! {
        <div class="screen">
            <header class="screen-header">
                <button class="back-btn" on:click=back>"‹"</button>
                <h1>"About"</h1>
            </header>

            <div class="about-content">
                <section class="about-section">
                    <h2>"Everything stays on your device"</h2>
                    <p>
                        "Your Codes are stored only in this browser's IndexedDB on this device. "
                        "There is no server, no cloud storage, and no account. Nothing is ever sent to us."
                    </p>
                </section>

                <section class="about-section">
                    <h2>"No logging, no tracking"</h2>
                    <p>
                        "This app runs entirely in your browser. It does not collect analytics, "
                        "does not log your activity, and does not share anything with anyone. "
                        "The service worker only caches the app's own files so it works offline."
                    </p>
                </section>

                <section class="about-section">
                    <h2>"Import and Export"</h2>
                    <p>
                        "Use Export to download a backup of your full Code collection as a JSON file. "
                        "Use Import to restore or add Codes from such a file — for example when you "
                        "move between a browser and a home-screen app. Already existing Codes are skipped."
                    </p>
                </section>

                <section class="about-section">
                    <h2>"Support this project"</h2>
                    <p>
                        "Barcode Wallet is free and open source. If you find it useful and want to "
                        "support its development, a small coffee is much appreciated."
                    </p>
                    <a class="btn about-support" href=KOFI_URL target="_blank" rel="noopener noreferrer">
                        "Support on Ko-fi"
                    </a>
                </section>
                <section class="about-section">
                    <h2>"License"</h2>
                    <p>
                    "Barcode wallet app" <br />
                    "Copyright (C) 2026 Bram Neijt"<br />
                    <br />
                    "This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version."
                    <br />
                    "This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details."
                    <br />
                    "You should have received a copy of the GNU Affero General Public License along with this program.  If not, see <http://www.gnu.org/licenses/>."
                    </p>
                </section>

                <p class="about-version">"Barcode Wallet v" {version}</p>
            </div>
        </div>
    }
}
