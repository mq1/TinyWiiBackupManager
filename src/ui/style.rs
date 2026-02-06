// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use iced::{
    Background, Color, Shadow, Theme, Vector, border,
    widget::{button, checkbox, container, text_input},
};

pub fn root_container(theme: &Theme) -> container::Style {
    let mut style = container::bordered_box(theme);
    style.border.radius = border::radius(0).top_left(10);
    style
}

pub fn rounded_button(theme: &Theme, status: button::Status) -> button::Style {
    let shadow_color = if theme.extended_palette().is_dark {
        Color::from_rgba8(100, 100, 100, 0.2)
    } else {
        Color::from_rgba8(0, 0, 0, 0.2)
    };

    let shadow = Shadow {
        color: shadow_color,
        offset: Vector::new(0.0, 0.0),
        blur_radius: 10.0,
    };

    let mut style = button::primary(theme, status);
    style.border.radius = border::radius(30);
    style.shadow = shadow;
    style
}

pub fn rounded_secondary_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::background(theme, status);
    style.border.width = 1.0;
    style.border.color = theme.extended_palette().background.strong.color;
    style.border.radius = border::radius(30);
    style
}

pub fn rounded_danger_button(theme: &Theme, status: button::Status) -> button::Style {
    let shadow_color = if theme.extended_palette().is_dark {
        Color::from_rgba8(100, 100, 100, 0.2)
    } else {
        Color::from_rgba8(0, 0, 0, 0.2)
    };

    let shadow = Shadow {
        color: shadow_color,
        offset: Vector::new(0.0, 0.0),
        blur_radius: 10.0,
    };

    let mut style = button::danger(theme, status);
    style.border.radius = border::radius(30);
    style.shadow = shadow;
    style
}

pub fn get_nav_button_style(active: bool) -> fn(&Theme, button::Status) -> button::Style {
    if active {
        active_nav_button
    } else {
        inactive_nav_button
    }
}

pub fn active_nav_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::background(theme, status);
    style.background = Some(Background::Color(theme.palette().primary.scale_alpha(0.5)));
    style.border.radius = border::radius(30);
    style
}

pub fn inactive_nav_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::background(theme, status);
    style.border.radius = border::radius(30);
    style
}

pub fn rounded_background_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::background(theme, status);
    style.border.radius = border::radius(30);
    style
}

pub fn nav_container(theme: &Theme) -> container::Style {
    let bg = theme.palette().background;
    container::background(bg)
}

pub fn card(theme: &Theme) -> container::Style {
    let bg = theme.palette().background;

    let shadow_color = if theme.extended_palette().is_dark {
        Color::from_rgba8(100, 100, 100, 0.1)
    } else {
        Color::from_rgba8(0, 0, 0, 0.1)
    };

    let shadow = Shadow {
        color: shadow_color,
        offset: Vector::new(0.0, 0.0),
        blur_radius: 10.0,
    };

    let mut style = container::bordered_box(theme);
    style.background = Some(Background::Color(bg));
    style.border.radius = border::radius(10);
    style.shadow = shadow;
    style
}

pub fn heavy_card(theme: &Theme) -> container::Style {
    let bg = theme.palette().background;

    let shadow_color = if theme.extended_palette().is_dark {
        Color::from_rgba8(100, 100, 100, 0.2)
    } else {
        Color::from_rgba8(0, 0, 0, 0.2)
    };

    let shadow = Shadow {
        color: shadow_color,
        offset: Vector::new(0.0, 0.0),
        blur_radius: 10.0,
    };

    let mut style = container::bordered_box(theme);
    style.background = Some(Background::Color(bg));
    style.border.radius = border::radius(10);
    style.shadow = shadow;
    style
}

pub fn search_bar(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let mut style = text_input::default(theme, status);
    style.background = Background::Color(Color::TRANSPARENT);
    style.border.width = 0.0;
    style
}

pub fn toolbar_checkbox(theme: &Theme, status: checkbox::Status) -> checkbox::Style {
    let mut style = checkbox::secondary(theme, status);
    style.border.radius = border::radius(30);
    style
}

pub fn rounded_text_input(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let mut style = text_input::default(theme, status);
    style.border.radius = border::radius(30);
    style
}
