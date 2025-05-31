use leptos::ev::MouseEvent;
use leptos::prelude::*;

#[component]
pub fn SubtitleLine(
    weather_data_ok: Option<bool>,
    handle_accept_config_btn_click: impl FnMut(MouseEvent) + 'static,
) -> impl IntoView {
    match weather_data_ok {
        Some(weather_data_ok) => {
            view! {
                <p style="color:rgb(186, 0, 192);">
                    "This is just a preview. Please "
                    {if weather_data_ok {
                        view! {
                            <a style="cursor: pointer;" on:click=handle_accept_config_btn_click>
                                "accept"
                            </a>
                            " to make it permanent, or "
                        }
                            .into_any()
                    } else {
                        ().into_any()
                    }} <a href="/">"discard"</a> " to restore your previous configuration."
                </p>
            }
        }
        .into_any(),
        _ => ().into_any(),
    }
}
