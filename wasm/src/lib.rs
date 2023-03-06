use chip8_core::*;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, KeyboardEvent};
use js_sys::Uint8Array;

mod input;

#[wasm_bindgen]
pub struct EmuWasm {
    chip8: Emu,
    ctx: CanvasRenderingContext2d
}

#[wasm_bindgen]
impl EmuWasm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<EmuWasm, JsValue> {
        let chip8 = Emu::new();

        let document = web_sys::window().unwrap()
            .document().unwrap();
        let canvas = document.query_selector("#canvas").unwrap().unwrap();
        let canvas: HtmlCanvasElement = canvas
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        let ctx = canvas.get_context("2d")
            .unwrap().unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();
        
        Ok(EmuWasm{chip8, ctx})
    }

    #[wasm_bindgen]
    pub fn tick(&mut self) {
        self.chip8.tick()
    }

    #[wasm_bindgen]
    pub fn tick_timers(&mut self) {
        self.chip8.tick_timers()
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.chip8.reset();
    }

    #[wasm_bindgen]
    pub fn keypress(&mut self, evt: KeyboardEvent, pressed: bool) {
        let key = evt.key();
        if let Some(k) = input::key2btn(&key) {
            self.chip8.keypress(k, pressed)
        }
    }

    #[wasm_bindgen]
    pub fn load_game(&mut self, data:Uint8Array) {
        self.chip8.load(&data.to_vec())
    }

    #[wasm_bindgen]
    pub fn draw_screen(&mut self, scale: usize) {
        let disp = self.chip8.get_screen();
        for i in 0..(SCREEN_HEIGHT*SCREEN_WIDTH) {
            if disp[i] {
                let x = i% SCREEN_WIDTH;
                let y = i / SCREEN_WIDTH;
                self.ctx.fill_rect(
                    (x*scale) as f64,
                    (y*scale) as f64,
                    scale as f64, 
                    scale as f64
                )
            }
        }
    }
}