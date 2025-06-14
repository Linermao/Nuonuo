use smithay::{
    backend::renderer::utils::on_commit_buffer_handler,
    delegate_compositor,
    reexports::wayland_server::{Client, protocol::wl_surface::WlSurface},
    wayland::compositor::{
        CompositorClientState, CompositorHandler, CompositorState, get_parent, is_sync_subsurface,
    },
};

use crate::state::{ClientState, GlobalData};

impl CompositorHandler for GlobalData {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.state.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        on_commit_buffer_handler::<Self>(surface);
        self.backend.early_import(surface);
        if !is_sync_subsurface(surface) {
            let mut root = surface.clone();
            while let Some(parent) = get_parent(&root) {
                root = parent;
            }

            if self.layer_shell_handle_commit(&root) {
                return;
            }

            if let Some(window) = self.workspace_manager.find_window(&root) {
                window.on_commit();
            }

            self.xdg_shell_handle_commit(surface);
            // resize_grab::handle_commit(&mut self.workspace_manager, surface);
        };
    }
}
delegate_compositor!(GlobalData);
