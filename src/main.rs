#![allow(non_snake_case, clippy::unwrap_used)]

use std::{any::Any, thread::sleep, time::Duration};

use base64::Engine;
use dioxus::{prelude::*, web::WebEventExt};
use dioxus_logger::tracing::{info, warn, Level};
use gloo_timers::callback::Timeout;
use image::buffer::ConvertBuffer;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    wasm_bindgen::{JsCast, UnwrapThrowExt},
    window, CanvasRenderingContext2d, HtmlCanvasElement, HtmlVideoElement, MediaStreamConstraints,
};

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/signup")]
    Signup,
}

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    info!("starting app");
    dioxus::launch(App);
}

fn App() -> Element {
    let tailwind_config = "tailwind.config = {
          theme: {
            extend: {
              colors: {
                background: '#101112',
                surface: '#202224',
              },
              fontFamily: {
                body: 'Fira Mono',
              },
            }
          }
        }";

    rsx! {


        head::Script { src: "https://cdn.tailwindcss.com" }
        head::Style {
            "@import url('https://fonts.googleapis.com/css2?family=Fira+Mono:wght@400;500;700&display=swap');"
        }
        // for manganis
        head::Link { rel: "stylesheet", href: asset!("./assets/tailwind.css") }
        head::Script {
            {tailwind_config}
        }
        div {
            class: "flex flex-col min-w-screen min-h-screen bg-background",
            Header {}
            div {
                class: "font-body",
                Router::<Route> {}
            }
        }
    }
}

#[component]
fn Signup() -> Element {
    rsx! {
        Link {
            to: Route::Signup,
            "Signup"
        }
    }
}

#[component]
fn Header() -> Element {
    rsx! {
        div {
            class: "grid grid-flow-col gap-16 items-center p-7 px-10 w-full text-red-100 font-body",
            div {
                class: "justify-self-start font-bold text-2xl",
                "Counterspell Develement Console"
            }
            div {
                class: "flex space-x-4 items-center justify-self-end text-lg",
                div {
                    "Signup"
                }
                div {
                    "Sadge"
                }
                div {
                    "Fuck"
                }
                div {
                    "Then"
                }
            }
        }
    }
}

#[component]
fn Nameplate(
    #[props(extends = div, extends = GlobalAttributes)] attributes: Vec<Attribute>,
    name: String,
    email: Option<String>,
    // children: Element,
) -> Element {
    rsx! {
        div {
            class: "w-96 rounded-xl aspect-[20/9] bg-surface p-6 px-8 hover:brightness-125",
            ..attributes,
            div {
                class: "grid grid-flow-col items-center h-full",
                if email.is_some() {
                    div {
                        class: "justify-self-start bg-white h-4/5 rounded-xl aspect-square",
                    }
                }
                div {
                    class: "text-white flex flex-col justify-center items-end gap-3",
                    class: if email.is_some() {
                        "justify-self-end"
                    } else {
                        "justify-self-center"
                    },
                    if let Some(email) = email.as_ref() {
                        div {
                            class: "text-3xl font-bold text-red-200",
                            {name}
                        }
                        div {
                            class: "text-sm",
                            {email.clone()}
                        }
                    } else {
                        div {
                            class: "text-4xl font-bold text-red-200",
                            {name}
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ArrowDown() -> Element {
    rsx! {
         svg { height: "40", xmlns: "http://www.w3.org/2000/svg", fill: "none", width: "40", "viewBox": "0 0 40 40", mask { "maskUnits": "userSpaceOnUse", style: "mask-type:alpha", x: "0", y: "0", width: "40", height: "40", id: "mask0_3567_51447", rect { width: "40", fill: "#D9D9D9", height: "40" } } g { mask: "url(#mask0_3567_51447)", path { fill: "#2F2F2F", d: "M19.1303 23.2824L13.1632 17.292C13.0707 17.1992 13.0042 17.1023 12.9637 17.0012C12.9231 16.9003 12.9028 16.7991 12.9028 16.6974C12.9028 16.4938 12.9795 16.3094 13.1328 16.1441C13.2859 15.9785 13.4852 15.8958 13.7307 15.8958H26.2607C26.5105 15.8958 26.7123 15.9799 26.8662 16.1483C27.0201 16.3163 27.097 16.5081 27.097 16.7237C27.097 16.7509 27.0089 16.9409 26.8328 17.2937L20.8695 23.2824C20.7589 23.393 20.6226 23.4794 20.4603 23.5416C20.2984 23.6038 20.1449 23.6349 19.9999 23.6349C19.8549 23.6349 19.7014 23.6038 19.5395 23.5416C19.3773 23.4794 19.2409 23.393 19.1303 23.2824Z", } } }
    }
}
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[allow(clippy::enum_variant_names)]
enum State {
    QRCodeScanning,
    InputUserID,
    WaitingForRFIDInsert,
    WriteToRFID,
    Finished,
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Size2D {
    pub width: f64,
    pub height: f64,
}

#[must_use]
pub fn use_mounted() -> Signal<Option<std::rc::Rc<MountedData>>> {
    use_signal(|| None)
}

#[component]
fn Camera(mut qr_code_content: Signal<Option<String>>) -> Element {
    let mut video: Signal<Option<HtmlVideoElement>> = use_signal(|| None);
    let mut canvas: Signal<Option<web_sys::HtmlCanvasElement>> = use_signal(|| None);
    let navigator = use_signal(|| window().unwrap().navigator());

    let _ = use_resource(move || {
        let runtime = Runtime::current().expect("Components run in the Dioxus runtime");
        async move {
            let _guard = RuntimeGuard::new(runtime);
            let constraints = MediaStreamConstraints::new();
            constraints.set_video(&true.into());
            let future = JsFuture::from(
                navigator()
                    .media_devices()
                    .unwrap()
                    .get_user_media_with_constraints(&constraints)
                    .unwrap(),
            );
            let stream = future.await.unwrap();
            if let Some(video) = video() {
                video.set_src_object(Some(&stream.dyn_into().unwrap()));
                let _ = video.play();
            }
        }
    });

    let mut video_ele = use_mounted();
    let mut width = use_signal(|| 0.);
    let mut height = use_signal(|| 0.);
    let mut err_code = use_signal(|| None);

    use_future(move || {
        let runtime = Runtime::current().expect("Components run in the Dioxus runtime");
        async move {
            loop {
                let _guard = RuntimeGuard::new(runtime.clone());
                if let (Some(canva), Some(video)) = (canvas(), video()) {
                    let current_size = if let Some(x) = &*video_ele.read() {
                        x.get_scroll_size()
                            .await
                            .map(|x| Size2D {
                                width: x.width,
                                height: x.height,
                            })
                            .unwrap_or_default()
                    } else {
                        Size2D::default()
                    };
                    width.set(current_size.width);
                    height.set(current_size.height);
                    // let rect = video.get_bounding_client_rect();
                    let (width, height) = (canva.width() as f64, canva.height() as f64);
                    if width != 0. && height != 0. {
                        let context = canva
                            .get_context("2d")
                            .unwrap()
                            .unwrap()
                            .dyn_into::<CanvasRenderingContext2d>()
                            .unwrap();
                        context
                            // .draw_image_with_html_video_element(&video, 0., 0.)
                            .draw_image_with_html_video_element_and_dw_and_dh(
                                &video, 0., 0., width, height,
                            )
                            .unwrap();

                        let base64ed = &canva.to_data_url_with_type("image/png").unwrap()[22..];

                        let standard = base64::engine::general_purpose::STANDARD;

                        let data = standard.decode(base64ed).unwrap();

                        if let Ok(img) =
                            image::load_from_memory_with_format(&data[..], image::ImageFormat::Png)
                        {
                            let img = img.to_luma8();
                            // Prepare for detection
                            let mut img = rqrr::PreparedImage::prepare(img);
                            // Search for grids, without decoding
                            let grids = img.detect_grids();
                            match grids.len() {
                                0 => {
                                    // nothing is detected, which can be normal
                                }
                                1 => {
                                    if let Ok((meta, content)) = grids[0].decode() {
                                        info!(content);
                                        qr_code_content.set(Some(content));
                                    } else {
                                        err_code.set(Some("try re-loacting your qr code"));
                                        warn!("error decoding string");
                                    }
                                }
                                _ => {
                                    warn!("somehow two or more qr code was detected?");
                                }
                            }
                        } else {
                            warn!("error loading image");
                        }
                    }
                }
                gloo_timers::future::sleep(Duration::from_millis(150)).await;
            }
        }
    });

    rsx! {
        if let Some(err_code) = err_code() {
            div {
                class: "text-2xl text-red-200 bg-surface rounded-xl p-3 w-fit",
                {err_code}
            }
        }
        canvas {
            class: "hidden",
            width: "{width()}px",
            height: "{height()}px",
            onmounted: move |event| canvas.set(event.data.as_web_event().dyn_into().ok()),
        }
        video {
            class: "w-full object-contain",
            onmounted: move |event| {
                video_ele.set(Some(event.data.clone()));
                video.set(event.data.as_web_event().dyn_into().ok());
            },
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct Info;

#[component]
fn Home() -> Element {
    let mut state = use_signal(|| State::QRCodeScanning);

    let qr_code_content = use_signal(|| None);

    let info = use_memo(move || qr_code_content.read().as_ref().map(|x| Info));

    use_effect(move || {
        if let Some(info) = &*info.read() {};
    });

    use_effect(move || {
        if qr_code_content.read().is_some() {
            state.set(State::WaitingForRFIDInsert);
        }
    });

    rsx! {
        div {
            class: "flex flex-col items-center h-full gap-8 px-16",
            div {
                class: "flex justify-center w-full mt-4",
                div {
                    class: "text-6xl text-white font-bold",
                    "Sign up Here!"
                }
            }
            div {
                class: "flex flex-col items-center gap-10 p-10",
                div {
                    class: "flex justify-center gap-5",
                    div {
                        class: "text-3xl font-bold p-3 pl-6 text-red-500",
                        match state() {
                            State::QRCodeScanning => {
                                "Please put your QRCode below the camera"
                            }
                            State::InputUserID => {
                                "Please input your user id"
                            }
                            State::WaitingForRFIDInsert => {
                                "Please insert your rfid"
                            }
                            State::WriteToRFID => {
                                "Please wait, writing to rfid"
                            }
                            State::Finished => {
                                "Congratulations! You have checked in successfully"
                            }
                        }
                    }
                }
                if matches!(state(), State::QRCodeScanning) {
                    div {
                        class: "flex flex-col gap-5",
                        Camera {
                            qr_code_content,
                        }
                        ManualInput { state }
                    }
                }
            }
        }
    }
}

#[component]
fn ManualInput(state: Signal<State>) -> Element {
    rsx! {
        div {
            class: "flex justify-end w-full p-5 pr-0",
            div {
                class: "transition-all text-yellow-100 bg-surface rounded-xl p-3 hover:brightness-125",
                onclick: move |_| {
                    state.set(State::InputUserID);
                },
                "Can't seem to scan it? try inputting manually"
            }
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    DownRightEdge,
    DownLeftEdge,
    UpRightEdge,
}

#[component]
pub fn ContextMenu(
    direction: Direction,
    children: Element,
    #[props(default = String::from("5px"))] gap: String,
) -> Element {
    let displacement = match direction {
        Direction::DownRightEdge => {
            format!("top:calc(100% + {gap}); right:0;")
        }
        Direction::UpRightEdge => {
            format!("top:0; left:calc(100% + {gap});")
        }
        Direction::DownLeftEdge => {
            format!("top:calc(100% + {gap}); left:0;")
        }
    };
    rsx! {
        div {
            class: "absolute border-2 border-surface w-fit h-fit rounded-xl *:rounded-xl *:p-5 z-[1000000] overflow-visible",
            style: "box-shadow: 10px 10px 30px 0px rgba(0, 0, 0, 0.25); {displacement}",
            {children}
        }
    }
}
