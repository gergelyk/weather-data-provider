use leptos::prelude::*;
use leptos::reactive::signal::WriteSignal;

#[component]
pub fn MessageDialog(
    set_message_dialog_is_open: WriteSignal<bool>,
    message_title: ReadSignal<String>,
    message_text: ReadSignal<String>,
) -> impl IntoView {
    view! {
        <dialog open>
            <article>
                <header>
                    <h3>{message_title}</h3>
                </header>
                <p inner_html=message_text />
                <footer>
                    <button
                        class="secondary"
                        on:click=move |_| {
                            set_message_dialog_is_open.set(false);
                        }
                    >
                        "Close"
                    </button>
                </footer>
            </article>
        </dialog>
    }
}
