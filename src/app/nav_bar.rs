use std::path::PathBuf;

use crate::app::{GlobalState, GlobalStateStoreFields, SelectedState};

use super::atoms::Icon;
use leptos::{html, logging::log, prelude::*, task::spawn_local};
use leptos_router::components::A;
use reactive_stores::Store;
use server_fn::codec::{MultipartData, MultipartFormData};
use wasm_bindgen::JsCast;
use web_sys::{Blob, Event, FormData, HtmlInputElement};

#[component]
pub fn NavBar() -> impl IntoView {
    view! {
        <nav class="flex flex-wrap">
            <Home/>
            <Clear/>
            <Download/>
            <Upload/>
            <Delete/>
            <Copy/>
            <Cut/>
            <Paste/>
        </nav>
    }
}

#[component]
fn Home() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let is_active = move || store.current_path().read().file_name().is_some();

    view! {
        <A href="/" class:disabled={move || !is_active()}>
            <Icon name="home" active={is_active}/>
        </A>
    }
}

#[component]
fn Clear() -> impl IntoView {
    let store = use_context::<Store<GlobalState>>().unwrap();
    let on_click = move |_| {
        store.select().write().clear();
    };

    let is_active = move || !store.select().read().is_clear();

    view! {
        <button
            disabled={move || !is_active()}
            on:click=on_click
        >
            <Icon active={is_active} name="clear"/>
        </button>
    }
}

#[server]
pub async fn rm(bases: Vec<PathBuf>) -> Result<(), ServerFnError> {
    use crate::ServerContext;
    use tokio::fs::remove_file;
    let context = use_context::<ServerContext>().unwrap();
    for base in bases.into_iter().map(|x| context.root.join(x)) {
        remove_file(base).await?;
    }

    Ok(())
}

#[component]
fn Delete() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let on_click = move |_| {
        spawn_local(async move {
            let result = rm(store.select().read_untracked().as_paths()).await;
            match result {
                Ok(_) => {
                    store.select().write().clear();
                    store.units_refetch_tick().update(|x| *x = !*x);
                }
                Err(e) => log!("Error : {:#?}", e),
            }
        });
    };

    let is_active = move || {
        let select = store.select().read();
        !select.is_clear() && !select.has_dirs()
    };

    view! {
        <button
            disabled={move || !is_active()}
            on:click=on_click
        >
            <Icon active={is_active} name="delete"/>
        </button>
    }
}

#[server]
pub async fn cp(from: Vec<PathBuf>, to: PathBuf) -> Result<(), ServerFnError> {
    use crate::ServerContext;
    use tokio::fs::copy;
    let context = use_context::<ServerContext>().unwrap();
    let to = context.root.join(to);
    for base in from.into_iter().map(|x| context.root.join(x)) {
        copy(&base, to.join(base.file_name().unwrap())).await?;
    }
    Ok(())
}

#[server]
pub async fn cp_cut(from: Vec<PathBuf>, to: PathBuf) -> Result<(), ServerFnError> {
    use crate::ServerContext;
    use tokio::fs::{copy, remove_file};
    let context = use_context::<ServerContext>().unwrap();
    let to = context.root.join(to);
    for base in from.into_iter().map(|x| context.root.join(x)) {
        copy(&base, to.join(base.file_name().unwrap())).await?;
        remove_file(base).await?;
    }
    Ok(())
}

#[component]
fn Paste() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let on_click = move |_| {
        spawn_local(async move {
            let to = store.current_path().get_untracked();
            let handle_result = |result| match result {
                Ok(_) => {
                    store.units_refetch_tick().update(|x| *x = !*x);
                }
                Err(e) => log!("Error : {:#?}", e),
            };
            match store.select().read().state {
                SelectedState::Copy => {
                    let result = cp(store.select().read_untracked().as_paths(), to).await;
                    handle_result(result);
                    store.select().write().clear();
                    store.select().write().none();
                }
                SelectedState::Cut => {
                    let result = cp_cut(store.select().read_untracked().as_paths(), to).await;
                    handle_result(result);
                    store.select().write().clear();
                    store.select().write().none();
                }
                SelectedState::None => (),
            }
        });
    };

    let is_active = move || {
        let select = store.select().read();
        !select.is_clear()
            && match select.state {
                SelectedState::Copy | SelectedState::Cut => true,
                SelectedState::None => false,
            }
    };

    view! {
        <button
            disabled={move || !is_active()}
            on:click=on_click
        >
            <Icon active={is_active} name="paste"/>
        </button>
    }
}

#[component]
fn Copy() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();

    let is_active = move || {
        let select = store.select().read();
        !select.is_clear() && !select.has_dirs()
    };

    let on_click = move |_| {
        store.select().write().copy();
    };

    view! {
        <button
            disabled={move || !is_active()}
            on:click=on_click
        >
            <Icon active={is_active} name="copy"/>
        </button>
    }
}

#[component]
fn Cut() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();

    let is_active = move || {
        let select = store.select().read();
        !select.is_clear() && !select.has_dirs()
    };

    let on_click = move |_| {
        store.select().write().cut();
    };

    view! {
        <button
            disabled={move || !is_active()}
            on:click=on_click
        >
            <Icon active={is_active} name="cut"/>
        </button>
    }
}

#[component]
fn Download() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();
    let on_click = move |_| {
        store.select().get_untracked().download_selected();
        store.select().write().clear();
    };

    let is_active = move || {
        let select = store.select().read();
        !select.is_clear() && !select.has_dirs()
    };

    view! {
        <button
            disabled={move || !is_active()}
            on:click=on_click
        >
            <Icon active={is_active} name="download"/>
        </button>
    }
}

#[server(
     input = MultipartFormData,
 )]
async fn upload(multipart: MultipartData) -> Result<(), ServerFnError> {
    use crate::ServerContext;
    use tokio::{
        fs::File,
        io::{AsyncWriteExt, BufWriter},
    };
    let context = use_context::<ServerContext>().unwrap();

    let mut data = multipart.into_inner().unwrap();

    while let Some(mut field) = data.next_field().await? {
        let path = context.root.join(field.name().unwrap().to_string());
        let mut file = BufWriter::new(File::create(path).await?);
        while let Some(chunk) = field.chunk().await? {
            file.write(&chunk).await?;
            file.flush().await?;
        }
    }

    Ok(())
}

#[component]
fn Upload() -> impl IntoView {
    let store: Store<GlobalState> = use_context().unwrap();

    let is_active = move || store.select().read().is_clear();

    let upload_action = Action::new_local(|data: &FormData| upload(data.clone().into()));
    let on_change = move |ev: Event| {
        ev.prevent_default();
        let target = ev
            .target()
            .unwrap()
            .unchecked_into::<HtmlInputElement>()
            .files()
            .unwrap();
        let data = FormData::new().unwrap();
        let mut i = 0;
        while let Some(file) = target.item(i) {
            let path = store.current_path().read().join(file.name());
            data.append_with_blob(path.to_str().unwrap(), &Blob::from(file))
                .unwrap();
            i += 1;
        }
        upload_action.dispatch_local(data);
    };
    let input_ref: NodeRef<html::Input> = NodeRef::new();

    let on_click = move |_| {
        input_ref.get().unwrap().click();
    };

    Effect::new(move || {
        if !upload_action.pending().get() {
            store.units_refetch_tick().update(|x| *x = !*x);
        }
    });

    view! {
        <Show
            when={move || !upload_action.pending().get()}
            fallback={move || view!{
                <img class="m-1 p-1" src="load.gif" width=65/>
            }}
            >
            <button
                disabled={move || !is_active()}
                on:click=on_click
            >
                <Icon active={is_active} name="upload"/>
            </button>
            <input node_ref={input_ref} on:change={on_change} type="file" multiple hidden/>
        </Show>
    }
}
