mod components;
mod config;
mod utils;
mod weather;

use anyhow::{Context, anyhow};
use components::{
    ConfigDialog, ImportDialog, MessageDialog, SubtitleLine, TitleLine, WeatherDataTable,
};
use config::{Config, download_config, get_local_config, set_local_config, upload_config};
use leptos::ev::MouseEvent;
use leptos::prelude::*;
use leptos::reactive::signal::WriteSignal;
use leptos::task::spawn_local;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use regex::Regex;
use utils::{get_root_url, log_anyhow_error};
use weather::get_weather_data;
use web_sys::{KeyboardEvent, window};

const LESMA_BASE_URL: &str = "https://lesma.eu";

fn build_config_id_regex() -> anyhow::Result<Regex> {
    let root = get_root_url().context("Cannot obtain root URL")?;
    let regex = format!(
        r"^(?:{}/|{}/preview/)?([a-z]+)$",
        regex::escape(LESMA_BASE_URL),
        regex::escape(&root)
    );
    let re = Regex::new(&regex)?;
    Ok(re)
}

fn load_config_into_signal(
    in_preview_mode: bool,
    config_id: &str,
    set_config: WriteSignal<Option<Result<Config, String>>>,
) {
    // console_log(&format!("in_preview_mode={}", in_preview_mode));
    // console_log(&format!("config_id={:?}", config_id));
    let config_id = config_id.to_owned();
    spawn_local({
        async move {
            if in_preview_mode {
                match download_config(&config_id).await {
                    Ok(config) => {
                        set_config.set(Some(Ok(config)));
                    }
                    Err(e) => {
                        let err_msg = e.to_string();
                        log_anyhow_error(e);
                        set_config.set(Some(Err(err_msg)));
                    }
                }
            } else {
                set_config.set(Some(Ok(get_local_config())));
            };
        }
    });
}

fn try_open_pb_service(id: &str) -> anyhow::Result<()> {
    let url = format!("{}/clone/{id}", LESMA_BASE_URL);
    if let Some(window) = window() {
        window
            .open_with_url_and_target(&url, "_blank")
            .map_err(|e| anyhow!(format!("{:?}", e)))?;
        Ok(())
    } else {
        Err(anyhow!("Failed to access the window"))
    }    
}

fn unwrap_config(
    config: ReadSignal<Option<Result<Config, String>>>,
) -> Result<Config, (anyhow::Error, String)> {
    let err_msg = "Failed to load config";
    let dialog_msg = "Sorry, we were unable to load this configuration.".to_owned();
    match config.get() {
        Some(Ok(config)) => Ok(config),
        Some(Err(e)) => Err((anyhow!(e).context(err_msg), dialog_msg)),
        None => Err((
            anyhow!("Configuration is None").context(err_msg),
            dialog_msg,
        )),
    }
}

#[component]
pub fn App(in_preview_mode: bool) -> impl IntoView {
    let navigate = use_navigate();
    let params = use_params_map();

    let (config_id, set_config_id) = signal(String::new());
    let (config, set_config) = signal::<Option<Result<Config, String>>>(None);

    Effect::new({
        move |_| {
            let new_config_id = params.read().get("id").unwrap_or_default();
            set_config_id.set(new_config_id);
        }
    });

    Effect::new({
        move |_| {
            load_config_into_signal(in_preview_mode, &config_id.get(), set_config);
        }
    });

    let weather_data = LocalResource::new(move || async move {
        match config.get() {
            Some(Ok(config)) => match get_weather_data(config).await {
                Ok((headers, measurements)) => Some(Ok((headers, measurements))),
                Err(e) => {
                    let err_msg = e.to_string();
                    log_anyhow_error(
                        anyhow!(err_msg.clone()).context("Failed to load weather data"),
                    );
                    Some(Err(format!("Failed to load weather data: {}", err_msg)))
                }
            },
            Some(Err(e)) => {
                log_anyhow_error(anyhow!(e.clone()).context("Failed to load config"));
                Some(Err(format!("Failed to load configuration: {:?}", e)))
            }
            None => None,
        }
    });

    let (config_dialog_is_open, set_config_dialog_is_open) = signal(false);
    let (import_dialog_is_open, set_import_dialog_is_open) = signal(false);
    let (message_dialog_is_open, set_message_dialog_is_open) = signal(false);

    let (message_title, set_message_title) = signal("".to_string());
    let (message_text, set_message_text) = signal("".to_string());
    let (import_src, set_import_src) = signal("".to_string()); // can be config ID or lema URL

    Effect::new(move |_| {
        if import_dialog_is_open.get() {
            set_import_src.set("".to_string());
        }
    });

    let show_message_dialog = move |title: &str, message: &str| {
        set_message_title.set(title.to_string());
        set_message_text.set(message.to_string());
        set_message_dialog_is_open.set(true);
    };

    let show_error_dialog = move |message: &str| {
        show_message_dialog("Error", message);
    };

    let handle_edit_btn_click = move |ev: MouseEvent| {
        ev.prevent_default();

        spawn_local(async move {
            let try_handle = async || {
                let config = unwrap_config(config)?;

                let err_msg = "Failed to edit config";
                let dialog_msg = "Sorry, we were unable to edit this configuration.".to_owned();

                let id = match upload_config(&config).await {
                    Ok(id) => id,
                    Err(e) => {
                        return Err((e.context(err_msg), dialog_msg));
                    }
                };

                if let Err(e) = try_open_pb_service(&id).context("Failed to open new tab") {
                    return Err((e.context(err_msg), dialog_msg));
                }

                set_import_dialog_is_open.set(true);
                Ok(())
            };

            try_handle().await.unwrap_or_else(|(e, dialog_msg)| {
                log_anyhow_error(e);
                show_error_dialog(&dialog_msg);
            });

            set_config_dialog_is_open.set(false);
        });
    };

    let handle_share_btn_click = move |ev: MouseEvent| {
        ev.prevent_default();

        spawn_local(async move {
            let try_handle = async || {
                let config = unwrap_config(config)?;

                let err_msg = "Failed to share config";
                let dialog_msg = "Sorry, we were unable to share this configuration.".to_owned();

                let id = match upload_config(&config).await {
                    Ok(id) => id,
                    Err(e) => {
                        return Err((e.context(err_msg), dialog_msg));
                    }
                };

                let root = match get_root_url() {
                    Ok(root) => root,
                    Err(e) => {
                        return Err((e.context(err_msg), dialog_msg));
                    }
                };

                let message = format!(
                    "Ask your friends to import this config ID: \
                    <big><b>{id}</b></big><br><br>\
                    or make them open this import URL: \
                    <b>{root}/preview/{id}</b><br><br>\
                    Both of them are valid for 24h."
                );
                show_message_dialog("Shared!", &message);

                Ok(())
            };

            try_handle().await.unwrap_or_else(|(e, dialog_msg)| {
                log_anyhow_error(e);
                show_error_dialog(&dialog_msg);
            });

            set_config_dialog_is_open.set(false);
        });
    };

    let handle_import_btn_click = move |ev: MouseEvent| {
        ev.prevent_default();
        set_config_dialog_is_open.set(false);
        set_import_dialog_is_open.set(true);
    };

    let handle_preview_submit = move || {
        match build_config_id_regex() {
            Ok(re) => match re.captures(&import_src.get().to_lowercase()) {
                Some(caps) => {
                    let config_id = caps[1].to_string();
                    let preview_url = format!("/preview/{}", config_id);
                    navigate(&preview_url, Default::default());
                    load_config_into_signal(in_preview_mode, &config_id, set_config);
                }
                None => {
                    let msg = "Invalid URL or config ID";
                    log_anyhow_error(anyhow!("{}: {}, ", msg, import_src.get()));
                    show_error_dialog(&format!("{}.", msg));
                }
            },
            Err(e) => {
                let msg = "Failed to generate preview URL";
                log_anyhow_error(e.context(msg));
                show_error_dialog(msg);
            }
        }
        set_import_dialog_is_open.set(false);
    };

    let handle_preview_btn_click = {
        let handle_preview_submit = handle_preview_submit.clone();
        move |ev: MouseEvent| {
            ev.prevent_default();
            handle_preview_submit();
        }
    };

    let handle_preview_import_src_keydown = {
        let handle_preview_submit = handle_preview_submit.clone();
        move |ev: KeyboardEvent| {
            if ev.key() == "Enter" {
                ev.prevent_default();
                handle_preview_submit();
            }
        }
    };

    let handle_accept_config_btn_click = {
        let navigate = use_navigate();
        move |ev: MouseEvent| {
            ev.prevent_default();
            if let Some(config) = config.get() {
                match config {
                    Ok(config) => {
                        set_local_config(&config);
                        navigate("/", Default::default());
                    }
                    Err(e) => {
                        log_anyhow_error(anyhow!(e).context("Failed to load config"));
                        show_error_dialog("Sorry, we were unable to load this configuration.");
                    }
                }
            }
        }
    };

    provide_meta_context();
    view! {
        <Title text="Weather Data" />

        <Show when=move || config_dialog_is_open.get()>
            <ConfigDialog
                set_config_dialog_is_open=set_config_dialog_is_open
                handle_edit_btn_click=handle_edit_btn_click
                handle_share_btn_click=handle_share_btn_click
                handle_import_btn_click=handle_import_btn_click
            />
        </Show>

        <Show when=move || import_dialog_is_open.get()>
            <ImportDialog
                import_src=import_src
                set_import_src=set_import_src
                set_import_dialog_is_open=set_import_dialog_is_open
                handle_preview_import_src_keydown=handle_preview_import_src_keydown.clone()
                handle_preview_btn_click=handle_preview_btn_click.clone()
            />
        </Show>

        <Show when=move || message_dialog_is_open.get()>
            <MessageDialog
                set_message_dialog_is_open=set_message_dialog_is_open
                message_title=message_title
                message_text=message_text
            />
        </Show>

        {move || {
            let (weather_data, weather_data_ok) = match *weather_data.read() {
                Some(Some(ref weather_data)) => {
                    (Some(weather_data.clone()), Some(weather_data.is_ok()))
                }
                _ => (None, None),
            };

            view! {
                <header class="container">
                    <hgroup>
                        <TitleLine set_config_dialog_is_open=set_config_dialog_is_open />
                        {if in_preview_mode {

                            view! {
                                <SubtitleLine
                                    weather_data_ok=weather_data_ok
                                    handle_accept_config_btn_click=handle_accept_config_btn_click
                                        .clone()
                                />
                            }
                                .into_any()
                        } else {
                            ().into_any()
                        }}
                    </hgroup>
                </header>

                <main class="container">
                    <WeatherDataTable weather_data=weather_data />
                </main>
            }
        }}
    }
}

pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! {
            <Router>
                <Routes fallback=|| "Not found">
                    <Route
                        path=leptos_router::path!("/")
                        view=|| view! { <App in_preview_mode=false /> }
                    />
                    <Route
                        path=leptos_router::path!("/preview/:id")
                        view=|| view! { <App in_preview_mode=true /> }
                    />
                </Routes>
            </Router>
        }
    })
}
