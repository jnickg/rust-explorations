extern crate base64;
use std::collections::HashMap;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use gloo::file::File;
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

/// A region of interest (ROI) in some target 2D coordinate space
#[derive(Clone, Copy, Debug)]
struct Roi2D {
    /// The top-left corner
    x: f64,
    /// The top-left corner
    y: f64,
    /// The width
    w: f64,
    /// The height
    h: f64,
}

#[derive(Clone, Copy, Debug)]
struct View2D {
    /// (x, y) - The _center_ of the view, in unit coordinates.
    ///
    /// (0.0, 0.0) is the top-left corner of the image, and (1.0, 1.0) is the bottom-right corner.
    unit_loc: (f64, f64),

    /// The zoom level of our view.
    zoom: f64,

    /// You have three guesses to figure out what this is for, and the first two guesses do not count.
    is_pan_active: bool,
}

#[derive(Clone, Copy, Debug)]
struct Dims {
    w: f64,
    h: f64,
}

#[derive(Clone, Copy, Debug)]
struct CanvasRoiPair {
    s: Roi2D,
    d: Roi2D,
}

/// Gets the pyramid level and re-scaled zoom factor, for the given effective zoom
///
/// 1.0 means full resolution, and 2.0 means we are zoomed in.
/// For every factor of half, we should increase pyramid level by 1. Anything above 1.0
/// should be considered as zoomed in.
///
/// Rescaled zoom means "relative zoom you need to use on the returned pyramid level to
/// achieve the given effective zoom"
fn level_and_relative_zoom_for(effective_zoom: f64) -> (u16, f64) {
    let effective_zoom = effective_zoom + f64::EPSILON; // Prevent dbz
                                                        // This computes the nearest _larger_ pyramid level, so that the browser only downsamples
                                                        // pyramid levels. We should only upsample on canvas when the user zooms into L0.
    let level = (1.0 / effective_zoom).log2().floor() as u16;
    // Compute how to achieve the desired effective zoom based on the level we've chosen
    let level_zoom = 0.5_f64.powi(level as i32);
    let relative_zoom = effective_zoom / level_zoom;
    (level, relative_zoom)
}

impl View2D {
    /// Convert the given view into a source ROI, and destination ROI
    ///
    /// This function accounts for relative aspect ratios and zoom level to determine the appropriate
    /// (sx, sy, sw, sh) and (dx, dy, dw, dh) values for the view.
    ///
    /// The source dimensions are the size of the actual image data to be sampled, not necessarily the
    /// original dimensions. That is, for low zoom levels the source dimensions may have been subsampled
    /// using a mipmap / image pyramid.
    ///
    /// The dest dimensions are the size of the canvas onto which the source is to be drawn. Most often
    /// these are scaled to the size of the original image, but that is not guaranteed on all viewports.
    ///
    /// This function accounts for mipmap subsampling by re-scaling the zoom factor to to be in terms of
    /// the image pyramid level that was used. If zoom is 0.5, the re-scaled zoom factor is 1.0, meaning
    /// the source image should be shown 1:1 in destination space. But if zoom is 0.6, the re-scaled zoom
    /// becomes 1.2, as the L1 pyramid was used, and that needs to be upsampled by 20% (6:5 ratio) in
    /// destination space to achieve a final effective zoom of 0.6.
    ///
    /// Finally, this function accounts for the aspect ratio of source and destination to determine what
    /// can (and can't) be shown in destination space. Based on the source dimensions and re-scaled zoom
    /// factor, we are able to determine whether the entire source image can be shown in the destination.
    /// If it can't, the source ROI is cropped to fit within the aspect ratio of the destination.
    ///
    /// This function ensures that [`View2D::unit_loc`], remains in the center of the destination
    ///
    /// # Returns
    /// A tuple of [`Roi2D`] structures
    ///
    /// # Notes
    /// - See: https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/drawImage
    ///   for explanation of values
    fn to_roi(
        &self,
        Dims { w: src_w, h: src_h }: Dims,
        Dims {
            w: dest_w,
            h: dest_h,
        }: Dims,
        use_relative_zoom: bool,
    ) -> CanvasRoiPair {
        let (_, relative_zoom) = if use_relative_zoom {
            level_and_relative_zoom_for(self.zoom)
        } else {
            (0u16, self.zoom)
        };
        web_sys::console::log_1(
            &format!("Relative zoom: {}, effective: {}", relative_zoom, self.zoom).into(),
        );
        web_sys::console::log_1(
            &format!(
                "Src dims: ({}, {}), dest dims: ({}, {})",
                src_w, src_h, dest_w, dest_h
            )
            .into(),
        );
        // Center of view in source image coordinates
        let csx = self.unit_loc.0 * src_w;
        let csy = self.unit_loc.1 * src_h;
        // ... scaled to dest space based on relative zoom
        let csxz = csx * relative_zoom;
        let csyz = csy * relative_zoom;
        // Center of the destination canvas (NOT the view). This is where we want to PUT the center
        // of the view
        let center_x_d = 0.5 * dest_w;
        let center_y_d = 0.5 * dest_h;
        // Origin of source image in destination space
        let sxd = center_x_d - csxz;
        let syd = center_y_d - csyz;
        // dx, dy need to get as close to sxd, syd as possible, within canvas bounds
        let dx = sxd.max(0.0).min(dest_w);
        let dy = syd.max(0.0).min(dest_h);
        // dw, dh can be computed based on the difference
        let swz = src_w * relative_zoom;
        let shz = src_h * relative_zoom;
        let dw = (swz - (dx - sxd)).min(dest_w).max(0.0);
        let dh = (shz - (dy - syd)).min(dest_h).max(0.0);
        // Offset from dest origin and source origin, in dest space
        let dxsz = dx - sxd;
        let dysz = dy - syd;
        // Now we can compute source ROI based on dest ROI offset
        let sx = (dxsz / relative_zoom).min(src_w).max(0.0);
        let sy = (dysz / relative_zoom).min(src_h).max(0.0);
        let sw = dw / relative_zoom;
        let sh = dh / relative_zoom;

        let result = CanvasRoiPair {
            s: Roi2D {
                x: sx,
                y: sy,
                w: sw,
                h: sh,
            },
            d: Roi2D {
                x: dx,
                y: dy,
                w: dw,
                h: dh,
            },
        };
        web_sys::console::log_1(&format!("ROIs: {:?}", result).into());
        result
    }
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
                // Example JSON (paste into separate doc and prettify)
                // {"image_docs":[{"$oid":"6660de9402834efab622c479"},{"$oid":"6660de9402834efab622c47a"},{"$oid":"6660de9402834efab622c47b"},{"$oid":"6660de9402834efab622c47c"},{"$oid":"6660de9402834efab622c47d"},{"$oid":"6660de9402834efab622c47e"},{"$oid":"6660de9402834efab622c47f"},{"$oid":"6660de9402834efab622c480"},{"$oid":"6660de9402834efab622c481"},{"$oid":"6660de9402834efab622c482"},{"$oid":"6660de9402834efab622c483"},{"$oid":"6660de9402834efab622c484"}],"image_files":[{"$oid":"6660de9402834efab622c460"},{"$oid":"6660de9402834efab622c463"},{"$oid":"6660de9402834efab622c465"},{"$oid":"6660de9402834efab622c467"},{"$oid":"6660de9402834efab622c469"},{"$oid":"6660de9402834efab622c46b"},{"$oid":"6660de9402834efab622c46d"},{"$oid":"6660de9402834efab622c46f"},{"$oid":"6660de9402834efab622c471"},{"$oid":"6660de9402834efab622c473"},{"$oid":"6660de9402834efab622c475"},{"$oid":"6660de9402834efab622c477"}],"image_names":["1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L0","1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L1","1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L2","1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L3","1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L4","1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L5","1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L6","1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L7","1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L8","1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L9","1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L10","1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L11"],"image_urls":["/api/v1/image/1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L0","/api/v1/image/1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L1","/api/v1/image/1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L2","/api/v1/image/1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L3","/api/v1/image/1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L4","/api/v1/image/1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L5","/api/v1/image/1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L6","/api/v1/image/1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L7","/api/v1/image/1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L8","/api/v1/image/1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L9","/api/v1/image/1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L10","/api/v1/image/1e1a4169-5bbf-4eed-bc3c-51a2a81d5221_L11"],"mime_type":"image/jpeg","tiles":"todo","url":"/api/v1/pyramid/1e1a4169-5bbf-4eed-bc3c-51a2a81d5221","uuid":"1e1a4169-5bbf-4eed-bc3c-51a2a81d5221"}
                //
                // Using that JSON structure we need to grab the `image_urls` and fetch the images then
                // send Msg::PyramidLevel for each image when received.
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
                let data_as_uint8array = Uint8Array::from(data.as_slice());
                let data_as_jsvalue = JsValue::from(data_as_uint8array);

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
                <div id="viewier-area">
                    <div class="info">
                        <p id="title">{ "Image Viewer" }</p>
                        <p>{ "Use the mouse wheel to zoom, and click and drag to pan" }</p>
                    </div>
                    <div class="content">
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
                                Msg::ViewPan((-event.movement_x() as f64, -event.movement_y() as f64))
                            })}
                        />
                    </div>
                </div>
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
        let (level, _) = level_and_relative_zoom_for(zoom);
        // If we have a cached pyramid level, use that instead
        if let Some(pyramid_id) = self.file_to_pyramid_id.get(&selected_image.name) {
            if let Some(pyramid_images) = self.pyramid_id_to_cached_pyramid_images.get(pyramid_id) {
                if let Some(image) = pyramid_images[level as usize].as_ref() {
                    web_sys::console::log_1(
                        &format!(
                            "Using cached pyramid level {} - dimensions: ({},{})",
                            level,
                            image.width(),
                            image.height()
                        )
                        .into(),
                    );
                    return Some(image);
                }
            }
        }

        // Fallback: If we couldn't find a cached pyramid level appropriate for the zoom, just
        // use the full-resolution loaded image.
        None
    }

    fn render_canvas(&self, _ctx: &Context<Self>) {
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
        let (image, use_relative_zoom) =
            match self.get_cached_image(selected_image_file_details, current_view.zoom) {
                Some(image) => (image, true),
                None => {
                    web_sys::console::log_1(&"Using full-resolution image".into());
                    (&selected_image_file_details.image, false)
                }
            };

        // Canvas should always be the same size as the original image
        canvas.set_width(selected_image_file_details.image.width());
        canvas.set_height(selected_image_file_details.image.height());

        let src_dims = Dims {
            w: image.width() as f64,
            h: image.height() as f64,
        };
        let dest_dims = Dims {
            w: canvas.width() as f64,
            h: canvas.height() as f64,
        };
        let CanvasRoiPair {
            s:
                Roi2D {
                    x: sx,
                    y: sy,
                    w: sw,
                    h: sh,
                },
            d:
                Roi2D {
                    x: dx,
                    y: dy,
                    w: dw,
                    h: dh,
                },
        } = current_view.to_roi(src_dims, dest_dims, use_relative_zoom);

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
