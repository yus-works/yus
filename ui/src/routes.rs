// ui/src/routes.rs
use leptos_router::components::Route;
use leptos_router::components::Routes;
use leptos_router::path;
use leptos::prelude::ElementChild;
use leptos::view;
use leptos::IntoView;
use leptos::component;

use crate::components::{
  demos_menu::DemosMenu,
  // demos::cube::CubeDemo,
};

use crate::pages::{
  classic::classic::ClassicMain,
  home::Home,
};

#[component]
pub fn RoutesMenu() -> impl IntoView {
    view! {
      <Routes fallback=|| view! { <p>"404 – not found"</p> }>
        <Route path=path!("")                view=Home        />
        <Route path=path!("/demos")          view=DemosMenu       />
        // <Route path=path!("/demos/mandelbrot") view=Mandelbrot/>
        // <Route path=path!("/demos/cube")     view=CubeDemo        />
        <Route path=path!("/classic")     view=ClassicMain        />
      </Routes>
    }
}
