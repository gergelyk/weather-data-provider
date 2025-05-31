use leptos::ev::MouseEvent;
use leptos::prelude::*;
use leptos::reactive::signal::WriteSignal;
use web_sys::KeyboardEvent;

#[component]
pub fn ImportDialog(
    import_src: ReadSignal<String>,
    set_import_src: WriteSignal<String>,
    set_import_dialog_is_open: WriteSignal<bool>,
    handle_preview_import_src_keydown: impl FnMut(KeyboardEvent) + 'static,
    handle_preview_btn_click: impl FnMut(MouseEvent) + 'static,
) -> impl IntoView {
    view! {
        <dialog open>
            <article>
                <header>
                    <h3>"Import Configuration"</h3>
                </header>
                <p>
                    "First, let's take a look. We won't override your current configuration until you accept the new one."
                </p>
                <p>
                    <input
                        name="import_src"
                        type="text"
                        placeholder="Config ID or lesma URL"
                        on:input:target=move |ev| {
                            set_import_src.set(ev.target().value());
                        }
                        on:keydown=handle_preview_import_src_keydown
                        autocomplete="off"
                        prop:value=import_src
                    />
                </p>
                <footer>
                    <button
                        class="secondary outline"
                        on:click=move |_| {
                            set_import_dialog_is_open.set(false);
                        }
                    >
                        "Cancel"
                    </button>
                    <button
                        class="secondary"
                        disabled=move || import_src.get().is_empty()
                        on:click=handle_preview_btn_click
                    >
                        "Preview"
                    </button>
                </footer>
            </article>
        </dialog>
    }
}
