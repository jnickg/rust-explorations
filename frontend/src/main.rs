extern crate base64;
use std::collections::HashMap;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use gloo::file::callbacks::FileReader;
use gloo::file::File;
use image::DynamicImage;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::{
    wasm_bindgen::JsCast, CanvasRenderingContext2d, DragEvent, Event, FileList, HtmlInputElement,
};
use yew::{html, Callback, Component, Context, Html, TargetCast, WheelEvent};

struct FileDetails {
    name: String,
    file_type: String,
    data: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
struct View2D {
    location: (f64, f64),
    zoom: f64,
}

pub enum Msg {
    Loaded(String, String, Vec<u8>),
    Files(Vec<File>),
    Pan((f64, f64)),
    Pyramid(String),
    PyramidTiles(String),
    Zoom(f64),
    SelectImage(String),
}

pub struct App {
    readers: HashMap<String, FileReader>,
    files: Vec<FileDetails>,
    file_to_pyramid_id: HashMap<String, String>,
    pyramid_ids: HashMap<String, Option<Vec<Vec<DynamicImage>>>>,
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
            pyramid_ids: HashMap::default(),
            selected_image: None,
            current_view: View2D {
                location: (0.0, 0.0),
                zoom: 1.0,
            },
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::PyramidTiles(pyramid_id) => {
                let tile_hierarchy: Vec<Vec<DynamicImage>> = Vec::default();
                // TODO fetch each tile, fetch them and put them into the tile hierarchy

                self.pyramid_ids
                    .insert(pyramid_id.clone(), Some(tile_hierarchy));
                true
            }
            Msg::Pyramid(pyramid_id) => {
                self.pyramid_ids.insert(pyramid_id.clone(), None);
                // TODO set up some background async poll for the pyramid's "tiles" state, and
                // when they are available, send the appropriate message so we know to render them
                // on screen.
                true
            }
            Msg::Loaded(file_name, file_type, data) => {
                self.files.push(FileDetails {
                    data,
                    file_type,
                    name: file_name.clone(),
                });
                self.readers.remove(&file_name);
                // TODO POST to backend the image to <hostname>:<port>/api/v1/pyramid with the
                // image data and mime type. Use a callback that handles the return of that
                // request, parses the JSON for the pyramid ID, and sends an appropriate message
                // like how we send link.send_message below

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
            Msg::Pan((dx, dy)) => {
                web_sys::console::log_1(&format!("Panning by: ({}, {})", dx, dy).into());
                self.current_view.location = (
                    self.current_view.location.0 + dx,
                    self.current_view.location.1 + dy,
                );
                self.render_canvas(ctx);
                true
            }
            Msg::Zoom(dz) => {
                self.current_view.zoom *= 1.0 + dz / 1000.0;
                self.render_canvas(ctx);
                true
            }
            Msg::SelectImage(file_name) => {
                // Console log
                web_sys::console::log_1(&format!("Selected image: {}", file_name).into());
                // Then set the selected image to the file name
                self.selected_image = Some(file_name);
                self.current_view = View2D {
                    location: (0.0, 0.0),
                    zoom: 1.0,
                };
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
                        Msg::Zoom(event.delta_y())
                    })}
                    ondrag={ctx.link().callback(|event: DragEvent| {
                        event.prevent_default();
                        Msg::Pan((event.movement_x() as f64, event.movement_y() as f64))
                    })}
                />
            </div>
        }
    }
}

impl App {
    fn get_canvas_ctx(
        &self,
    ) -> Result<
        (
            web_sys::HtmlCanvasElement,
            web_sys::CanvasRenderingContext2d,
        ),
        (),
    > {
        let canvas = web_sys::window()
            .ok_or(())?
            .document()
            .ok_or(())?
            .get_element_by_id("viewer-canvas")
            .ok_or(())?
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())?;
        let ctx = canvas
            .get_context("2d")
            .map_err(|_| ())?
            .ok_or(())?
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .map_err(|_| ())?;
        Ok((canvas, ctx))
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
        let image_data = &selected_image_file_details.data;
        // createImageBitmap is not available in web-sys yet, so we have to use the
        // Image constructor instead
        let image = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("img")
            .unwrap();
        image
            .set_attribute(
                "src",
                &format!(
                    "data:{};base64,{}",
                    selected_image_file_details.file_type,
                    STANDARD.encode(image_data)
                ),
            )
            .unwrap();
        let image = image.dyn_into::<web_sys::HtmlImageElement>().unwrap();
        canvas.set_width(image.width());
        canvas.set_height(image.height());
        let current_view = self.current_view;
        // See: https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/drawImage
        // This is what we'd do by default:
        //   let dx = 0.0;
        //   let dy = 0.0;
        //   let dw = canvas.width() as f64;
        //   let dh = canvas.height() as f64;
        //   let sx = 0.0;
        //   let sy = 0.0;
        //   let sw = image.width() as f64;
        //   let sh = image.height() as f64;
        // But we want to pan and zoom, so we need to adjust the values
        let dx = current_view.location.0;
        let dy = current_view.location.1;
        let dw = canvas.width() as f64 * current_view.zoom;
        let dh = canvas.height() as f64 * current_view.zoom;
        let sx = 0.0;
        let sy = 0.0;
        let sw = image.width() as f64;
        let sh = image.height() as f64;

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
