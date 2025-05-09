use smithay::{
    backend::renderer::{damage::OutputDamageTracker, element::{surface::WaylandSurfaceRenderElement, RenderElement}, gles::GlesRenderer, ImportAll, ImportMem, Renderer},
    desktop::space::render_output,
    output::Output,
};

#[cfg(feature = "tty")]
use crate::backend::tty::{Tty, TtyRenderer};
use crate::{backend::winit::Winit, state::GlobalData};
use crate::manager::workspace::Workspace;

use super::elements::{OutputRenderElements, WindowRenderElement};
use super::{
    border::BorderShader,
    cursor::{RenderCursor, XCursor},
    elements::CustomRenderElements,
};

// TODO:
impl GlobalData {
    // pub fn get_render_elements<R>(&self, renderer: R) -> Vec<OutputRenderElements<R, WindowRenderElement<R>>>
    //     where 
    //         R: Renderer + ImportAll + ImportMem,
    //         R::TextureId: Clone + 'static,
    // {
    //     let mut output_render_elements: Vec<OutputRenderElements<R, WindowRenderElement<R>>> = vec![];
    //     // let custom_elements: Vec<CustomRenderElements> = self.get_render_elements();
    //     let output = self.output_manager.current_output();
    //     let space = &self.workspace_manager.current_workspace().space;

    //     let elements: Vec<smithay::desktop::space::SpaceRenderElements<R, WindowRenderElement<R>>> = space.render_elements_for_output(renderer, output, 1.0).unwrap();

    //     output_render_elements.extend(
    //         elements.into_iter().map(OutputRenderElements::Space)
    //     );

    //     output_render_elements
    // }

    // // TODO: fix this, only use winit renderer
    // pub fn get_render_elements(&mut self) -> Vec<CustomRenderElements> {
    //     let mut custom_elements: Vec<CustomRenderElements> = vec![];
    //     // add pointer elements
    //     custom_elements.extend(self.get_cursor_render_elements());
    //     // // add window's border
    //     // custom_elements.extend(self.get_border_render_elements());
    //     // // add windows
    //     // custom_elements.extend(
    //     //     self.get_windows_render_elements()
    //     // );
    //     custom_elements
    // }

    // pub fn _get_windows_render_elements(&mut self) -> Vec<CustomRenderElements> {
    //     let (renderer, _) = self.backend.winit().backend.bind().unwrap();
    //     let output = self.output_manager.current_output();
    //     let output_geometry = self
    //         .workspace_manager
    //         .current_workspace()
    //         .space
    //         .output_geometry(output)
    //         .unwrap();
    //     let output_scale = self.output_manager.current_output().current_scale();
    //     let output_scale = Scale::from(output_scale.fractional_scale());

    //     self.workspace_manager
    //         .current_workspace()
    //         .elements()
    //         .rev()
    //         .flat_map(|w| {
    //             let loc = (self
    //                 .workspace_manager
    //                 .current_workspace()
    //                 .space
    //                 .element_geometry(w)
    //                 .unwrap()
    //                 .loc
    //                 - output_geometry.loc
    //                 - w.geometry().loc)
    //                 .to_physical_precise_round(output_scale);
    //             w.render_elements(renderer, loc, output_scale, 1.0)
    //         })
    //         .collect()
    // }

    // pub fn get_cursor_render_elements(&mut self) -> Vec<CustomRenderElements> {
    //     self.cursor_manager.check_cursor_image_surface_alive();

    //     let output_scale = self.output_manager.current_output().current_scale();
    //     let output_pos = self
    //         .workspace_manager
    //         .current_workspace()
    //         .space
    //         .output_geometry(self.output_manager.current_output())
    //         .unwrap()
    //         .loc;

    //     let pointer_pos = self.seat.get_pointer().unwrap().current_location();
    //     let pointer_pos = pointer_pos - output_pos.to_f64();

    //     let cursor_scale = output_scale.integer_scale();
    //     let render_cursor = self.cursor_manager.get_render_cursor(cursor_scale);

    //     let output_scale = Scale::from(output_scale.fractional_scale());

    //     let pointer_render_elements: Vec<CustomRenderElements> = match render_cursor {
    //         RenderCursor::Hidden => vec![],
    //         RenderCursor::Surface { hotspot, surface } => {
    //             let real_pointer_pos =
    //                 (pointer_pos - hotspot.to_f64()).to_physical_precise_round(output_scale);

    //             render_elements_from_surface_tree(
    //                 self.backend.winit().backend.renderer(),
    //                 &surface,
    //                 real_pointer_pos,
    //                 output_scale,
    //                 1.0,
    //                 Kind::Cursor,
    //             )
    //         }
    //         RenderCursor::Named {
    //             icon,
    //             scale,
    //             cursor,
    //         } => {
    //             let (idx, frame) = cursor.frame(self.start_time.elapsed().as_millis() as u32);
    //             let hotspot = XCursor::hotspot(frame).to_logical(scale);
    //             let pointer_pos =
    //                 (pointer_pos - hotspot.to_f64()).to_physical_precise_round(output_scale);

    //             let texture = self.cursor_texture_cache.get(icon, scale, &cursor, idx);
    //             let mut pointer_elements = vec![];
    //             let pointer_element = match MemoryRenderBufferRenderElement::from_buffer(
    //                 self.backend.winit().backend.renderer(),
    //                 pointer_pos,
    //                 &texture,
    //                 None,
    //                 None,
    //                 None,
    //                 Kind::Cursor,
    //             ) {
    //                 Ok(element) => Some(element),
    //                 Err(err) => {
    //                     warn!("error importing a cursor texture: {err:?}");
    //                     None
    //                 }
    //             };
    //             if let Some(element) = pointer_element {
    //                 pointer_elements.push(CustomRenderElements::NamedPointer(element));
    //             }
    //             pointer_elements
    //         }
    //     };
    //     pointer_render_elements
    // }

    // pub fn get_border_render_elements(&mut self) -> Vec<CustomRenderElements> {
    //     let mut elements: Vec<CustomRenderElements> = vec![];

    //     let focus_surface = self.seat.get_keyboard().unwrap().current_focus();
    //     if let Some(surface) = focus_surface {
            
    //         let focused_window = self
    //             .workspace_manager
    //             .current_workspace()
    //             .space
    //             .elements()
    //             .find(|w| *w.toplevel().unwrap().wl_surface() == surface);

    //         if let Some(window) = focused_window {
    //             let geometry = self
    //                 .workspace_manager
    //                 .current_workspace()
    //                 .space
    //                 .element_geometry(window)
    //                 .unwrap();
    //             elements.push(CustomRenderElements::Border(BorderShader::element(
    //                 &self.backend.winit().backend.renderer(),
    //                 geometry,
    //                 1.0,
    //             )));
    //         }
    //     }

    //     elements
    // }
}

impl Winit {
    pub fn render_output(&mut self, output: &Output, workspace: &Workspace, custom_elements: Vec<CustomRenderElements<GlesRenderer>>) {

        let (renderer, mut framebuffer) = self.backend.bind().unwrap();

        let damage_tracker = &mut OutputDamageTracker::from_output(
            output,
        );

        render_output::<_, CustomRenderElements<GlesRenderer>, _, _>(
            output,
            renderer,
            &mut framebuffer,
            1.0,
            0,
            [&workspace.space],
            custom_elements.as_slice(),
            damage_tracker,
            [0.0, 0.0, 1.0, 1.0],
        )
        .unwrap();
    }
}

#[cfg(feature = "tty")]
impl Tty {
    pub fn render_output(&mut self, output: &Output, workspace: &Workspace, custom_elements: Vec<CustomRenderElements<TtyRenderer>>) {
    
    }
}