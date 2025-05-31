use leptos::prelude::*;

#[component]
pub fn TitleLine(set_config_dialog_is_open: WriteSignal<bool>) -> impl IntoView {
    view! {
        <nav class="nav-flex">
            <h1>"Weather Data"</h1>
            <ul>
                <li>
                    <img
                        class="icon"
                        src="/static/gear.svg"
                        style="cursor: pointer; height:1em; vertical-align:top;"
                        on:click=move |_| {
                            set_config_dialog_is_open.set(true);
                        }
                    />
                </li>
            </ul>
        </nav>
    }
}
