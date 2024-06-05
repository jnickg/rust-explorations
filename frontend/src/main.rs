extern crate base64;
use std::collections::HashMap;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use gloo::file::File;
use gloo::timers::future;
use gloo::{file::callbacks::FileReader, utils::format::JsValueSerdeExt};
use js_sys::Uint8Array;
use wasm_bindgen::JsValue;
use web_sys::HtmlImageElement;
use web_sys::{
    wasm_bindgen::JsCast, CanvasRenderingContext2d, DragEvent, Event, FileList, HtmlCanvasElement,
    HtmlInputElement, Request, Response,
};
use yew::{html, Callback, Component, Context, Html, MouseEvent, TargetCast, WheelEvent};

struct FileDetails {
    name: String,
    file_type: String,
    data: Vec<u8>,
    image: HtmlImageElement,
}

#[derive(Clone, Copy, Debug)]
struct View2D {
    /// (x, y) - The location of our view over an image, in unit coordinates.
    ///
    /// (0.0, 0.0) is the top-left corner of the image, and (1.0, 1.0) is the bottom-right corner.
    unit_loc: (f64, f64),

    /// The zoom level of our view.
    zoom: f64,
    is_pan_active: bool,
}

impl Default for View2D {
    fn default() -> Self {
        Self {
            unit_loc: (0.5, 0.5),
            zoom: 1.0,
            is_pan_active: false,
        }
    }
}

pub enum Msg {
    /// An image has been loaded into memory
    ///
    /// file_name, file_type, data
    Loaded(String, String, Vec<u8>),
    Files(Vec<File>),
    ViewPan((f64, f64)),
    ViewPanState(bool),
    /// A new pyramid has been created
    ///
    /// pyramid_id, file_name, pyramid_json
    Pyramid(String, String, serde_json::Value),
    /// A new pyramid level is available for the given pyramid
    ///
    /// (pyramid_id, pyramid_level, file_type, data)
    PyramidLevel(String, u8, String, Vec<u8>),
    ViewZoom(f64),
    SelectImage(String),
}

pub struct App {
    readers: HashMap<String, FileReader>,
    files: Vec<FileDetails>,
    file_to_pyramid_id: HashMap<String, String>,
    pyramid_id_to_cached_pyramid_images: HashMap<String, Vec<Option<HtmlImageElement>>>,
    pyramid_id_to_json: HashMap<String, serde_json::Value>,
    selected_image: Option<String>,
    current_view: View2D,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            readers: HashMap::default(),
            files: Vec::default(),
            file_to_pyramid_id: HashMap::default(),
            pyramid_id_to_cached_pyramid_images: HashMap::default(),
            pyramid_id_to_json: HashMap::default(),
            selected_image: None,
            current_view: View2D::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Pyramid(pyramid_id, file_name, pyramid_json) => {
                web_sys::console::log_2(
                    &"Received pyramid ID and JSON".into(),
                    &pyramid_json.to_string().into(),
                );
                self.file_to_pyramid_id
                    .insert(file_name.clone(), pyramid_id.clone());
                self.pyramid_id_to_json
                    .insert(pyramid_id.clone(), pyramid_json.clone());
                // LONG TERM:
                // We need to use some kind of cache system to fetch (and delete) pyramid-level images
                // based on the user's current view. We can't just fetch all the images at once, because
                // that would be a lot of data to transfer.
                //
                // And, continuously poll pyramid/<pyramid_id> to check for tiles. When THOSE are available
                // cache them and use them instead of the pyramid images.
                //
                // SHORT TERM:
                // Fetch all the pyramid level images and cache them locally when available.
                //
                //
                // Example JSON:
                // {"image_docs":[{"$oid":"6660de9402834efab622c479"},{"$oid":"6660de9402834efab622c47a"},{"$oid":"6660de9402834efab622c47b"},{"$oid":"6660de9402834efab622c47c"},{"$oid":"6660de9402834efab622c47d"},{"$oid":"6660de9402834efab622c47e"},{"$oid":"6660de9402834efab622c47f"},{"$oid":"6660de9402834efab622c480"},{"$oid":"6660de9402834efab622c481"},{"$oid":"6660de9402834efab622c482"},{"$oid":"6660de9402834efab622c483"},{"$oid":"6660de9402834efab622c484"}],"image_files":[{"$oid":"6660de9402834efab622c460"},{"$oid":"6660de9402834efab622c463"},{"$oid":"6660de9402834efab622c465"},{"$oid":"6660de9402834efab622c467"},{"$oid":"6660de9402834efab622c469"},{"$oid":"6660de9402834efab622c46b"},{"$oid":"6660de9402834efab622c46d"},{"$oid":"6660de9402834efab622c46f"},{"$oid":"6660de9402834efab622c471"},{"$oid":"6660de9402834efab622c473"},{"$oid":"6660de9402834efab622c475"},{"$oid":"6660de9402834efab622c477"}],"image_names":["1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L0","1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L1","1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L2","1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L3","1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L4","1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L5","1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L6","1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L7","1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L8","1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L9","1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L10","1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L11"],"image_urls":["/api/v1/image/1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L0","/api/v1/image/1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L1","/api/v1/image/1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L2","/api/v1/image/1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L3","/api/v1/image/1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L4","/api/v1/image/1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L5","/api/v1/image/1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L6","/api/v1/image/1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L7","/api/v1/image/1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L8","/api/v1/image/1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L9","/api/v1/image/1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L10","/api/v1/image/1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L11"],"mime_type":"image/jpeg","tiles":"todo","url":"/api/v1/pyramid/1e1a4169-5bbf-4eed-bc3c-51a2a81d5221","uuid":"1e1a4169-5bbf-4eed-bc3c-51a2a81d5221"}
                //
                // Using that JSON, grab the image_urls and fetch the images, send Msg::PyramidLevel for each
                // image.
                let image_urls = pyramid_json.get("image_urls").unwrap().as_array().unwrap();
                self.pyramid_id_to_cached_pyramid_images
                    .insert(pyramid_id.clone(), vec![None; image_urls.len()]);
                let window = match web_sys::window() {
                    Some(window) => window,
                    None => {
                        web_sys::console::log_1(&"Failed to get window".into());
                        return false;
                    }
                };
                for (i, image_url) in image_urls.iter().enumerate() {
                    let image_url = image_url.as_str().unwrap();
                    let request = Request::new_with_str(image_url).unwrap();
                    let link = ctx.link().clone();
                    let pyramid_id = pyramid_id.clone();
                    let file_name = file_name.clone();
                    let pyramid_level = i as u8;
                    let future =
                        wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request));
                    wasm_bindgen_futures::spawn_local(async move {
                        match future.await {
                            Ok(response) => {
                                let response = response
                                    .dyn_into::<Response>()
                                    .expect("Failed to convert response");
                                let ab_promise = response.array_buffer().unwrap();
                                let ab = wasm_bindgen_futures::JsFuture::from(ab_promise)
                                    .await
                                    .unwrap();
                                let data = js_sys::Uint8Array::new(&ab).to_vec();
                                let file_type = response.headers().get("Content-Type").unwrap();
                                link.send_message(Msg::PyramidLevel(
                                    pyramid_id,
                                    pyramid_level,
                                    file_type.unwrap(),
                                    data,
                                ));
                            }
                            Err(e) => {
                                web_sys::console::log_1(&format!("Error fetching: {:?}", e).into());
                            }
                        }
                    });
                }
                true
            }
            Msg::PyramidLevel(pyramid_id, pyramid_level, file_type, data) => {
                let mut pyramid_images = self
                    .pyramid_id_to_cached_pyramid_images
                    .get_mut(&pyramid_id);
                let pyramid_images = pyramid_images.as_mut().unwrap();
                let image = HtmlImageElement::new().unwrap();
                image.set_src(&format!(
                    "data:{};base64,{}",
                    file_type,
                    STANDARD.encode(data.as_slice())
                ));
                pyramid_images[pyramid_level as usize] = Some(image);
                true
            }
            Msg::Loaded(file_name, file_type, data) => {
                // analogous curl:
                //      `curl --data-binary "@helldivers.jpg" -H "Content-Type: image/jpeg" -X POST http://localhost:3000/api/v1/pyramid`
                let data_as_jsvalue = JsValue::from(Uint8Array::from(data.as_slice()));
                let request = Request::new_with_str_and_init(
                    "http://localhost:8080/api/v1/pyramid",
                    &web_sys::RequestInit::new()
                        .method("POST")
                        .body(Some(&data_as_jsvalue)),
                )
                .unwrap();
                request
                    .headers()
                    .set("Content-Type", &file_type.clone())
                    .unwrap();
                let link = ctx.link().clone();
                match web_sys::window() {
                    Some(window) => {
                        let fetch = window.fetch_with_request(&request);
                        let future = wasm_bindgen_futures::JsFuture::from(fetch);
                        let file_name_local = file_name.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            match future.await {
                                Ok(response) => {
                                    let response = response
                                        .dyn_into::<Response>()
                                        .expect("Failed to convert response");
                                    let json_promise = response.json().unwrap();
                                    let json = wasm_bindgen_futures::JsFuture::from(json_promise)
                                        .await
                                        .unwrap();
                                    let pyramid_json =
                                        json.into_serde::<serde_json::Value>().unwrap();
                                    // get the "uuid" field from the JSON
                                    let pyramid_id = pyramid_json
                                        .get("uuid")
                                        .unwrap()
                                        .as_str()
                                        .unwrap()
                                        .to_string();

                                    link.send_message(Msg::Pyramid(
                                        pyramid_id,
                                        file_name_local,
                                        pyramid_json,
                                    ));
                                }
                                Err(e) => {
                                    web_sys::console::log_1(
                                        &format!("Error fetching: {:?}", e).into(),
                                    );
                                }
                            }
                        });
                    }
                    None => {
                        web_sys::console::log_1(&"Failed to get window".into());
                    }
                }

                let image = HtmlImageElement::new().unwrap();
                image
                    .set_attribute(
                        "src",
                        &format!(
                            "data:{};base64,{}",
                            file_type.clone(),
                            STANDARD.encode(data.clone())
                        ),
                    )
                    .unwrap();

                self.files.push(FileDetails {
                    data,
                    file_type: file_type.clone(),
                    name: file_name.clone(),
                    image,
                });
                self.readers.remove(&file_name);

                true
            }
            Msg::Files(files) => {
                for file in files.into_iter() {
                    let file_name = file.name();
                    let file_type = file.raw_mime_type();

                    let task = {
                        let link = ctx.link().clone();
                        let file_name = file_name.clone();

                        gloo::file::callbacks::read_as_bytes(&file, move |res| {
                            link.send_message(Msg::Loaded(
                                file_name,
                                file_type,
                                res.expect("failed to read file"),
                            ))
                        })
                    };
                    self.readers.insert(file_name, task);
                }
                true
            }
            Msg::ViewPan((dx, dy)) => {
                if !self.current_view.is_pan_active {
                    return false;
                }
                let (x_unit, y_unit) = self.current_view.unit_loc;
                let dx_unit =
                    dx / self.current_view.zoom / self.get_canvas_ctx().unwrap().0.width() as f64;
                let dy_unit =
                    dy / self.current_view.zoom / self.get_canvas_ctx().unwrap().0.height() as f64;
                let x_unit = (x_unit + dx_unit).max(0.0).min(1.0);
                let y_unit = (y_unit + dy_unit).max(0.0).min(1.0);
                self.current_view.unit_loc = (x_unit, y_unit);

                self.render_canvas(ctx);
                true
            }
            Msg::ViewPanState(is_panning) => {
                self.current_view.is_pan_active = is_panning;
                true
            }
            Msg::ViewZoom(dz) => {
                self.current_view.zoom *= 1.0 + dz / 1000.0;
                self.render_canvas(ctx);
                true
            }
            Msg::SelectImage(file_name) => {
                web_sys::console::log_1(&format!("Selected image: {}", file_name).into());
                self.selected_image = Some(file_name);
                // Delete the pyramid-level images that we don't need anymore, and cache the
                // pyramid-level images for the currently-selected image.
                self.current_view = View2D::default();
                self.render_canvas(ctx);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div id="wrapper">
                <p id="title">{ "Load file(s)" }</p>
                <label for="file-upload">
                    <div
                        id="drop-container"
                        ondrop={ctx.link().callback(|event: DragEvent| {
                            event.prevent_default();
                            let files = event.data_transfer().unwrap().files();
                            Self::upload_files(files)
                        })}
                        ondragover={Callback::from(|event: DragEvent| {
                            event.prevent_default();
                        })}
                        ondragenter={Callback::from(|event: DragEvent| {
                            event.prevent_default();
                        })}
                    >
                        <i class="fa fa-cloud-upload"></i>
                        <p>{"Drop your images here or click to select"}</p>
                    </div>
                </label>
                <input
                    id="file-upload"
                    type="file"
                    accept="image/*"
                    onchange={ctx.link().callback(move |e: Event| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        Self::upload_files(input.files())
                    })}
                />
                <p id="title">{ "Select an image" }</p>
                <p>{ "Click on an image to view it in the viewer, then scroll down to view it." }</p>
                <div id="preview-area">
                    { for self.files.iter().map(
                        |file| self.preview_file(ctx, file)
                    ) }
                </div>
                <p id="title">{ "Image Viewer" }</p>
                <p>{ "Use the mouse wheel to zoom, and click and drag to pan" }</p>
                <canvas
                    id="viewer-canvas"
                    onwheel={ctx.link().callback(|event: WheelEvent| {
                        event.prevent_default();
                        Msg::ViewZoom(-event.delta_y())
                    })}
                    onmousedown={ctx.link().callback(|_| Msg::ViewPanState(true))}
                    onmouseup={ctx.link().callback(|_| Msg::ViewPanState(false))}
                    onmouseleave={ctx.link().callback(|_| Msg::ViewPanState(false))}
                    onmousemove={ctx.link().callback(|event: MouseEvent| {
                        event.prevent_default();
                        Msg::ViewPan((event.movement_x() as f64, event.movement_y() as f64))
                    })}
                />
            </div>
        }
    }
}

impl App {
    fn get_canvas_ctx(&self) -> Result<(HtmlCanvasElement, CanvasRenderingContext2d), ()> {
        let canvas = web_sys::window()
            .ok_or(())?
            .document()
            .ok_or(())?
            .get_element_by_id("viewer-canvas")
            .ok_or(())?
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| ())?;
        let ctx = canvas
            .get_context("2d")
            .map_err(|_| ())?
            .ok_or(())?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| ())?;
        Ok((canvas, ctx))
    }

    fn get_cached_image(
        &self,
        selected_image: &FileDetails,
        zoom: f64,
    ) -> Option<&HtmlImageElement> {
        // TODO based on zoom level, we should be able to select the appropriate level of the
        // pyramid to render, rather than grabbing the L0 (full resolution) image every time.
        let level_from_zoom = |zoom: f64| -> u8 {
            // 1.0 means full resolution, and 2.0 means we are zoomed in.
            // For every factor of half, we should increase pyramid level by 1. Anything above 1.0
            // should be considered as zoomed in.
            let mut level = 0;
            let mut zoom = zoom;
            while zoom < 1.0 {
                zoom *= 2.0;
                level += 1;
            }
            level
        };
        let level = level_from_zoom(zoom);
        // If we have a cached pyramid level, use that instead
        if let Some(pyramid_id) = self.file_to_pyramid_id.get(&selected_image.name) {
            if let Some(pyramid_images) = self.pyramid_id_to_cached_pyramid_images.get(pyramid_id) {
                if let Some(image) = pyramid_images[level as usize].as_ref() {
                    web_sys::console::log_1(
                        &format!("Using cached pyramid level {}", level).into(),
                    );
                    return Some(image);
                }
            }
        }

        // Fallback: If we couldn't find a cached pyramid level appropriate for the zoom, just
        // use the full-resolution loaded image.
        None
    }

    fn render_canvas(&self, ctx: &Context<Self>) {
        let (canvas, canvas_ctx) = match self.get_canvas_ctx() {
            Ok((canvas, ctx)) => (canvas, ctx),
            Err(_) => return,
        };

        // Clear the canvas
        canvas_ctx.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);

        // Draw the image
        let selected_image = if let Some(selected_image) = self.selected_image.as_ref() {
            selected_image
        } else {
            // Draw a placeholder
            canvas_ctx.set_fill_style(&"black".into());
            canvas_ctx.fill_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
            return;
        };

        let selected_image_file_details =
            match self.files.iter().find(|file| file.name == *selected_image) {
                Some(file) => file,
                None => return,
            };

        let current_view = self.current_view;
        let image = match self.get_cached_image(selected_image_file_details, current_view.zoom) {
            Some(image) => image,
            None => {
                web_sys::console::log_1(&"Using full-resolution image".into());
                &selected_image_file_details.image
            }
        };

        canvas.set_width(selected_image_file_details.image.width());
        canvas.set_height(selected_image_file_details.image.height());
        // See: https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/drawImage
        // This is what we'd do for typical pan/zoom
        //   let (dx, dy) = current_view.location;
        //   let dw = canvas.width() as f64 * current_view.zoom;
        //   let dh = canvas.height() as f64 * current_view.zoom;
        //   let sx = 0.0;
        //   let sy = 0.0;
        //   let sw = image.width() as f64;
        //   let sh = image.height() as f64;
        // But we use the unit location instead, and that point represents the _center_ of the view,
        // so we need to adjust the dx and dy accordingly.
        let (dx_unit, dy_unit) = current_view.unit_loc;
        let dw = canvas.width() as f64 * current_view.zoom;
        let dh = canvas.height() as f64 * current_view.zoom;
        let sx = 0.0;
        let sy = 0.0;
        let sw = image.width() as f64;
        let sh = image.height() as f64;
        let dx = (dx_unit - 0.5) * sw * current_view.zoom;
        let dy = (dy_unit - 0.5) * sh * current_view.zoom;

        match canvas_ctx
            .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &image, sx, sy, sw, sh, dx, dy, dw, dh,
            ) {
            Ok(_) => {}
            Err(e) => {
                web_sys::console::log_1(&format!("Error drawing image: {:?}", e).into());
            }
        }
    }

    fn preview_file(&self, ctx: &Context<Self>, file: &FileDetails) -> Html {
        let is_selected = self
            .selected_image
            .as_ref()
            .map_or(false, |selected| selected == &file.name);
        let class_str = if is_selected {
            "preview-tile selected"
        } else {
            "preview-tile"
        };
        html! {
            <div
                class={class_str}
                onclick={
                    let file_name = file.name.clone();
                    ctx.link().callback(move |_| {
                        Msg::SelectImage(file_name.clone())
                    })
                }
            >
                <p class="preview-name">{ format!("{}", file.name) }</p>
                <div class="preview-media">
                    if file.file_type.contains("image") {
                        <img src={format!("data:{};base64,{}", file.file_type, STANDARD.encode(&file.data))} />
                    }
                </div>
            </div>
        }
    }

    fn upload_files(files: Option<FileList>) -> Msg {
        let mut result = Vec::new();

        if let Some(files) = files {
            let files = js_sys::try_iter(&files)
                .unwrap()
                .unwrap()
                .map(|v| web_sys::File::from(v.unwrap()))
                .map(File::from);
            result.extend(files);
        }
        Msg::Files(result)
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
