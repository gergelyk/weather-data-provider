use leptos::ev::MouseEvent;
use leptos::prelude::*;
use leptos::reactive::signal::WriteSignal;

#[component]
pub fn ConfigDialog(
    set_config_dialog_is_open: WriteSignal<bool>,
    handle_edit_btn_click: impl FnMut(MouseEvent) + 'static,
    handle_share_btn_click: impl FnMut(MouseEvent) + 'static,
    handle_import_btn_click: impl FnMut(MouseEvent) + 'static,
) -> impl IntoView {
    view! {
        <dialog open>
            <article>
                <header>
                    <h3>"Configuration"</h3>
                </header>

                <p class="grid">
                    <button on:click=handle_edit_btn_click>"Edit"</button>
                </p>

                <p class="grid">
                    <button on:click=handle_share_btn_click>"Share"</button>
                </p>

                <p class="grid">
                    <button on:click=handle_import_btn_click>"Import"</button>
                </p>

                <footer>
                    <button
                        class="secondary"
                        on:click=move |_| {
                            set_config_dialog_is_open.set(false);
                        }
                    >
                        "Close"
                    </button>
                </footer>
            </article>
        </dialog>
    }
}
