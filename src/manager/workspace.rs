use std::sync::atomic::{AtomicUsize, Ordering};

use smithay::{
    desktop::{Space, Window},
    output::Output,
    reexports::{wayland_protocols::xdg::shell::server::xdg_toplevel, wayland_server::protocol::wl_surface::WlSurface},
    utils::{Logical, Point, Rectangle},
};

use crate::layout::{tiled_tree::{LayoutScheme, TiledTree}, LayoutHandle};


static NEXT_ID: AtomicUsize = AtomicUsize::new(1);
const GAP: i32 = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkspaceId(usize);

impl WorkspaceId {
    // only for test
    pub fn new (id: usize) -> Self {
        Self(id)
    }
    #[inline]
    pub fn next() -> Self {
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub space: Space<Window>,
    pub layout: LayoutScheme,
    pub layout_tree: Option<TiledTree>,
    pub focus: Option<Window>,
}

impl Workspace {
    pub fn new(output: &Output, location: (i32, i32), layout: LayoutScheme) -> Self {
        let mut space: Space<Window> = Default::default();
        space.map_output(output, location);
        Self {
            id: WorkspaceId::next(),
            space,
            layout,
            layout_tree: None,
            focus: None,
        }
    }

    pub fn id(&self) -> WorkspaceId {
        self.id
    }

    pub fn map_element(&mut self, window: Window, location: Point<i32, Logical>, activate: bool) {
        self.space.map_element(window, location, activate);
    }

    pub fn map_tiled_element(
        &mut self,
        window: Window,
        output: &Output,
        focused_surface: Option<WlSurface>,
        activate: bool,
    ) {
        self.refresh();
        let output_geo = self.output_geometry(output);

        if self.layout_tree.is_none() {
            window.toplevel().unwrap().with_pending_state(|state| {
                state.size = Some(output_geo.size - (GAP*2, GAP*2).into())
            });
            window.toplevel().unwrap().send_pending_configure();
            window.set_rec(Rectangle::new((GAP, GAP).into(), (output_geo.size - (GAP*2, GAP*2).into()).into()));
            self.layout_tree = Some(TiledTree::new(window.clone()));
            self.map_element(window, (GAP, GAP).into(), activate);
            return;
        }

        let focus = focused_surface.and_then(|surface| {
            self.elements().find(|win| *win.toplevel().unwrap().wl_surface() == surface)
        })
        .unwrap()
        .clone();

        match self.layout {
            LayoutScheme::Default => {
                if let Some(layout_tree) = &mut self.layout_tree {
                    layout_tree.insert_window(&focus, window.clone());

                    #[cfg(feature="trace_layout")]
                    layout_tree.print_tree();
                    let loc = window.get_rec().unwrap().loc;
                    self.map_element(window, loc, activate);
                }
            },
            LayoutScheme::BinaryTree => {
                todo!()
            }
        }
        self.send_pending_configure();
    }

    pub fn unmap_tiled_element(&mut self, window: Window) {
        if let Some(layout_tree) = &mut self.layout_tree {
            layout_tree.remove(&window);

            if layout_tree.is_empty() {
                self.layout_tree = None;
            } else {
                #[cfg(feature="trace_layout")]
                layout_tree.print_tree();
            }
        } else {
            panic!("empty layout tree!");
        }
    }

    pub fn raise_element(&mut self, window: &Window, activate: bool) {
        self.space.raise_element(window, activate);
    }

    pub fn refresh(&mut self) {
        self.space.refresh();
    }

    pub fn element_under(
        &self,
        position: Point<f64, Logical>,
    ) -> Option<(&Window, Point<i32, Logical>)> {
        self.space.element_under(position).map(|(w, p)| (w, p))
    }

    pub fn element_location(&self, window: &Window) -> Point<i32, Logical> {
        self.space.element_location(window).unwrap()
    }

    pub fn element_geometry(&self, window: &Window) -> Rectangle<i32, Logical> {
        self.space.element_geometry(window).unwrap()
    }

    pub fn elements(&self) -> impl DoubleEndedIterator<Item = &Window> + ExactSizeIterator {
        self.space.elements()
    }

    // TODO: move this to outputmanager
    pub fn output_geometry(&self, output: &Output) -> Rectangle<i32, Logical> {
        self.space.output_geometry(output).unwrap()
    }

    // deactivate all window
    pub fn deactivate(&mut self) {
        for window in self.space.elements() {
            window.set_activated(false);
            window.toplevel().unwrap().send_pending_configure();
        }
    }
    
    pub fn remove_window(&mut self, surface: &WlSurface){

        let window = self.elements().find(|win| {
            win.toplevel().unwrap().wl_surface() == surface
        })
        .unwrap()
        .clone();

        self.space.unmap_elem(&window);
        self.unmap_tiled_element(window);

        self.send_pending_configure();
    }

    pub fn invert_window(&mut self, focused_surface: Option<WlSurface>) {
        if self.layout_tree.is_none() || focused_surface.is_none() {
            return;
        }
        let focus = focused_surface.and_then(|surface| {
            self.elements().find(|win| *win.toplevel().unwrap().wl_surface() == surface)
        })
        .unwrap()
        .clone();

        if let Some(layout_tree) = &mut self.layout_tree {
            layout_tree.invert_window(&focus);

            #[cfg(feature="trace_layout")]
            layout_tree.print_tree();

            self.send_pending_configure();
        }
    }

    pub fn modify_windows(&mut self, rec: Rectangle<i32, Logical>) {
        if let Some(layout_tree) = &mut self.layout_tree {
            let root_id = layout_tree.get_root().unwrap();
            layout_tree.modify(root_id, Rectangle::new((GAP, GAP).into(), (rec.size - (GAP*2, GAP*2).into()).into()));
        }
        self.send_pending_configure();
    }
    
    pub fn resize(&mut self, focused_surface: Option<WlSurface>, offset: (i32, i32)) {
        if self.layout_tree.is_none() || focused_surface.is_none() {
            return;
        }
        
        self.send_resize_configure();

        let focus = focused_surface.and_then(|surface| {
            self.elements().find(|win| *win.toplevel().unwrap().wl_surface() == surface)
        })
        .unwrap()
        .clone();

        if let Some(layout_tree) = &mut self.layout_tree {
            layout_tree.resize(&focus, offset);
        }

        self.send_unresize_configure();
    }

    pub fn send_resize_configure(&mut self) {
        for win in self.elements() {
            win.toplevel().unwrap().with_pending_state(|state| {
                state.states.set(xdg_toplevel::State::Resizing)
            });
            win.toplevel().unwrap().send_pending_configure();
        }
    }

    pub fn send_unresize_configure(&mut self) {
        let e: Vec<_> = self.elements().cloned().collect();
        
        for win in e {
            let rec = win.get_rec().unwrap();
            win.toplevel().unwrap().with_pending_state(|state| {
                state.states.unset(xdg_toplevel::State::Resizing);
                state.size = Some(rec.size)
            });
            win.toplevel().unwrap().send_pending_configure();
            self.map_element(win, rec.loc, false);
        }
    }
    
    pub fn send_pending_configure(&mut self) {
        let e: Vec<_> = self.elements().cloned().collect();
        
        for win in e {
            let rec = win.get_rec().unwrap();
            win.toplevel().unwrap().with_pending_state(|state| {
                state.size = Some(rec.size)
            });
            win.toplevel().unwrap().send_pending_configure();
            self.map_element(win, rec.loc, false);
        }
    }

    pub fn set_focus(&mut self, surface: WlSurface) {
        let focus_window = self
            .elements()
            .find(|w| *w.toplevel().unwrap().wl_surface() == surface);

        if let Some(window) = focus_window {
            self.focus = Some(window.clone())
        }
    }

    pub fn get_focus(&self) -> &Option<Window> {
        &self.focus
    }

}

#[derive(Debug)]
pub struct WorkspaceManager {
    pub workspaces: Vec<Workspace>,
    pub activated_workspace: Option<WorkspaceId>,
}

impl WorkspaceManager {
    pub fn new() -> Self {
        Self {
            workspaces: vec![],
            activated_workspace: None,
        }
    }

    // TODO: allow more output binds
    pub fn add_workspace(
        &mut self,
        output: &Output,
        location: (i32, i32),
        layout: Option<LayoutScheme>,
        activate: bool,
    ) {
        let workspace = Workspace::new(
            output,
            location,
            layout.unwrap_or_else(|| LayoutScheme::Default),
        );

        if activate {
            self.set_activated(workspace.id());
        }

        self.workspaces.push(workspace);
    }

    pub fn set_activated(&mut self, workspace_id: WorkspaceId) {
        if let Some(id) = self.activated_workspace {
            if id != workspace_id {
                self.current_workspace_mut().deactivate();
                self.activated_workspace = Some(workspace_id);
            }
        } else {
            self.activated_workspace = Some(workspace_id);
        }
    }

    pub fn _remove_workspace(&mut self, _workspace_id: usize) {
        // move windows
        todo!()
    }

    pub fn current_workspace(&self) -> &Workspace {
        self.activated_workspace
            .and_then(|id| self.workspaces.iter().find(|w| w.id() == id))
            .expect("no current_workspace")
    }

    pub fn current_workspace_mut(&mut self) -> &mut Workspace {
        self.activated_workspace
            .and_then(|id| self.workspaces.iter_mut().find(|w| w.id() == id))
            .expect("no current_workspace")
    }

    pub fn map_element(&mut self, window: Window, location: Point<i32, Logical>, activate: bool) {
        self.current_workspace_mut()
            .map_element(window, location, activate);
    }

    pub fn map_tiled_element(
        &mut self,
        window: Window,
        output: &Output,
        focused_surface: Option<WlSurface>,
        activate: bool,
    ) {
        self.current_workspace_mut().map_tiled_element(window, output, focused_surface, activate);
    }

    pub fn raise_element(&mut self, window: &Window, activate: bool) {
        self.current_workspace_mut().raise_element(window, activate);
    }

    pub fn refresh(&mut self) {
        self.current_workspace_mut().refresh();
    }

    pub fn element_under(
        &self,
        position: Point<f64, Logical>,
    ) -> Option<(&Window, Point<i32, Logical>)> {
        self.current_workspace()
            .element_under(position)
            .map(|(w, p)| (w, p))
    }

    pub fn element_location(&self, window: &Window) -> Point<i32, Logical> {
        self.current_workspace().element_location(window)
    }

    pub fn element_geometry(&self, window: &Window) -> Rectangle<i32, Logical> {
        self.current_workspace().element_geometry(window)
    }

    pub fn elements(&self) -> impl DoubleEndedIterator<Item = &Window> + ExactSizeIterator {
        self.current_workspace().elements()
    }

    pub fn find_window(&self, wl_surface: &WlSurface) -> &Window {
        // TODO: maybe can use hashmap to store the surface
        self.elements()
            .find(|w| w.toplevel().unwrap().wl_surface() == wl_surface)
            .unwrap()
    }

    pub fn output_geometry(&self, output: &Output) -> Rectangle<i32, Logical> {
        self.current_workspace().output_geometry(output)
    }

    pub fn _workspaces_counts(&self) -> usize {
        self.workspaces.iter().count()
    }
 
    pub fn remove_window(&mut self, surface: &WlSurface) {
        self.current_workspace_mut().remove_window(surface);
    }

    pub fn invert_window(&mut self, focused_surface: Option<WlSurface>) {
        self.current_workspace_mut().invert_window(focused_surface);
    }

    pub fn modify_windows(&mut self, rec: Rectangle<i32, Logical>) {
        self.current_workspace_mut().modify_windows(rec);
    }

    pub fn resize(&mut self, focused_surface: Option<WlSurface>, offset: (i32, i32)) {
        self.current_workspace_mut().resize(focused_surface, offset);
    }

    pub fn set_focus(&mut self, surface: WlSurface) {
        self.current_workspace_mut().set_focus(surface);
    }

}
