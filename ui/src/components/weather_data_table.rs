use crate::weather::{CellValue, WeatherData};
use leptos::prelude::*;

#[component]
pub fn WeatherDataTable(weather_data: Option<Result<WeatherData, String>>) -> impl IntoView {
    view! {
        <section>
            {match weather_data {
                Some(Ok((headers, measurements))) => {
                    view! {
                        <>
                            <div class="overflow-auto">
                                <table class="striped">
                                    <thead>
                                        <tr>
                                            {headers
                                                .iter()
                                                .map(|(title, _)| {
                                                    view! { <th>{title.as_str()}</th> }
                                                })
                                                .collect_view()}
                                        </tr>
                                    </thead>
                                    <tr>
                                        {headers
                                            .iter()
                                            .map(|(_, unit)| {
                                                view! {
                                                    <td>
                                                        {if unit.is_empty() {
                                                            ().into_any()
                                                        } else {
                                                            view! { <>"["{unit.as_str()}"]"</> }.into_any()
                                                        }}

                                                    </td>
                                                }
                                            })
                                            .collect_view()}
                                    </tr>
                                    <tbody>
                                        {measurements
                                            .iter()
                                            .map(|row| {
                                                view! {
                                                    <tr>
                                                        {row
                                                            .iter()
                                                            .map(|value| {
                                                                view! {
                                                                    <td>
                                                                        {match value {
                                                                            CellValue::Text(text) => {
                                                                                view! { <>{text.clone()}</> }.into_any()
                                                                            }
                                                                            CellValue::Link(text, href) => {
                                                                                view! { <a href=href.clone()>{text.clone()}</a> }.into_any()
                                                                            }
                                                                            CellValue::NotAvailable => {
                                                                                {
                                                                                    view! { <small style="color: gray;">N/A</small> }
                                                                                }
                                                                                    .into_any()
                                                                            }
                                                                        }}

                                                                    </td>
                                                                }
                                                            })
                                                            .collect_view()}

                                                    </tr>
                                                }
                                            })
                                            .collect_view()}
                                    </tbody>
                                </table>
                            </div>
                        </>
                    }
                        .into_any()
                }
                Some(Err(e)) => {
                    view! {
                        <>
                            <h2 style="color: #C00000;">"Error:"</h2>
                            {e}
                        </>
                    }
                        .into_any()
                }
                None => view! { <>"Loading..."</> }.into_any(),
            }}
        </section>
    }
}
