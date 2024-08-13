use maud::{html, Markup, PreEscaped, Render, DOCTYPE};
use crate::routes::api::request_auth_snippet;

// root page layout
pub fn root(contents: PreEscaped<String>) -> String {
    let html_res = html! {
        (head())
        (navbar())
        #body-contents {
            (contents)
        }
        (footer())
    };

    html_res.into_string()
}
pub fn head() -> Markup {
    html! {
            (DOCTYPE)
            html lang="en";
            head {
                meta charset="utf-8";
                link rel="stylesheet" type="text/css" href="/static/style.css";
                title { "OpenComputer Access Terminal" }
            }
    }
}
pub fn footer() -> Markup {
    html! {
        script src="https://unpkg.com/htmx.org@1.9.12" {}
        script src="/static/script.js" {}
    }
}
pub(crate) fn navbar() -> Markup {
    assert!(
        !&BUTTONS.is_empty(),
        "Navigation bar should have buttons (none were loaded from nav_buttons())"
    );

    let last_button = &BUTTONS.last().unwrap();
    html! {
        div ."navbar bg-base-100" hx-boost="true" hx-target="#body-contents" hx-push-url="true" {
            // some icon here instead..
            div ."flex-none" {
                button ."btn btn-square btn-ghost" {
                    svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        class="inline-block h-5 w-5 stroke-current" {
                        path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M4 6h16M4 12h16M4 18h16" {}
                    }
                }
            }

            @for button in &BUTTONS[0 .. &BUTTONS.len()-1] {
                a ."btn btn-ghost text-xl"
                    href=(button.url) { (button.name) }
            }
            div ."flex-1" {
                a ."btn btn-ghost text-xl" hx-target="#body-contents" hx-push-url="true"
                    href=(last_button.url) { (last_button.name) }
            }

            div ."flex items-stretch" {
                button class="btn btn-info rounded-btn" onclick={"request_authorization_modal.showModal()"} { "+" }
                div ."dropdown dropdown-end" {
                    div tabindex="0" role="button" class="btn btn-ghost rounded-btn" {
                        svg
                            xmlns="http://www.w3.org/2000/svg"
                            fill="none"
                            viewBox="0 0 24 24"
                            class="inline-block h-5 w-5 stroke-current" {
                            path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M5 12h.01M12 12h.01M19 12h.01M6 12a1 1 0 11-2 \
                                0 1 1 0 012 0zm7 0a1 1 0 11-2 0 1 1 0 012 0zm7 0a1 \
                                1 0 11-2 0 1 1 0 012 0z" {}
                        }
                    }

                    ul tabindex="0" class="menu \
                    dropdown-content bg-base-100 rounded-box z-[1] mt-4 w-52 p-2 shadow" {
                        @for button in &DROPDOWN_BUTTONS {
                            li { a hx-get=(button.url) { (button.name) } }
                        }

                    }
                }
            }
        }
        (Modal::new("request_authorization_modal",
            "Generate a new authorization key:",
            request_auth_snippet()))
    }
}

struct Modal {
    id: &'static str,
    title: &'static str,
    body: Markup,
}

impl Modal {
    pub fn new(id: &'static str, title: &'static str, body: Markup) -> Modal {
        Modal { id, title, body }
    }
}

impl Render for Modal {
    fn render(&self) -> Markup {
        html! {
            dialog id=(self.id) class="modal" {
                div class="modal-box" {
                    h3 class="text-lg font-bold text-center" { (self.title) }
                    p class="py-4" { (self.body) }
                }

                form method="dialog" class="modal-backdrop" {
                    button { "close" }
                }
            }
        }
    }
}

pub struct Link {
    pub(crate) name: &'static str,
    pub(crate) url: &'static str,
}

impl Link {
    pub const fn new(name: &'static str, url: &'static str) -> Link {
        Link { name, url }
    }
}

pub static BUTTONS: [Link; 5] = [
    Link::new("Home", "/"),
    Link::new("Clients", "/clients"),
    Link::new("Energy", "/energy"),
    Link::new("Items", "/items"),
    Link::new("Statistics", "/stats"),
];

pub static DROPDOWN_BUTTONS: [Link; 2] = [
    Link::new("Unknown", "/seals"),
    Link::new("Settings", "/settings"),
];