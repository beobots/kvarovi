use std::convert::Infallible;
use leptos::*;
use leptos::server_fn::serde;
use serde::{Deserialize, Serialize};


#[derive(Deserialize)]
struct Streets {
    streets: Vec<Street>,
}

#[derive(Deserialize)]
struct Street {
    name: String,
}

const STREET_SOURCE: &str = if let Some(url) = option_env!("BEO_STREETS_URL") { url } else { "http://localhost:3000/failure.json" };

async fn load_data() -> Result<Streets, gloo::net::Error> {
    let response = gloo::net::http::Request::get(STREET_SOURCE).send().await?;
    let result = response.json::<Streets>().await?;
    Ok(result)
}

#[component]
fn App(cx: Scope) -> impl IntoView {
    let (count, set_count) = create_signal(cx, 0);
    view! { cx,
        <button
            class:red={move || count.get() % 2 == 0 }
            on:click=move |_| {
                set_count.set(count.get() + 1);
            }
        >
            "Increment value: "
            {move || count.get()}
        </button>
    }
}

async fn loading_data() -> Result<String, Infallible> {
    let _x = gloo::timers::future::Timeout::new(1_000).await;
    web_sys::console::log_1(&"timeout has passed".into());
    Ok("this is data from the remote server...".to_string())
}

fn main() {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&"mapapp is starting...".into());

    let async_data = create_resource(
        cx,
        || (),
        |value| async move { loading_data().await.map_err(|_| "unknown error".to_string()) },
    );

    let async_result = move || {
        async_data
            .read(cx)
            .map(|value| format!("Server returned {value:?}"))
            .unwrap_or_else(|| "Loading...".into())
    };

    mount_to_body(|cx| {
        view! { cx,  <div><App />
            <p>
            testing...
                {async_result}
            </p></div>
        }
    })
}