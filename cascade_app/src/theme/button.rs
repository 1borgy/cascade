use iced::{
    widget::button::{Catalog, Status, Style, StyleFn},
    Background, Border, Color,
};

use crate::theme::Theme;

//impl Catalog for Theme {
//    type Class<'a> = StyleFn<'a, Self>;
//
//    fn default<'a>() -> Self::Class<'a> {
//        Box::new(default)
//    }
//
//    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
//        class(self, status)
//    }
//}
//
//fn default(theme: &Theme, status: Status) -> Style {
//    primary(theme, status, false)
//}
//
//fn button(
//    foreground: Color,
//    background: Color,
//    background_hover: Color,
//    status: Status,
//) -> Style {
//    match status {
//        Status::Active | Status::Pressed => Style {
//            background: Some(Background::Color(background)),
//            text_color: foreground,
//            border: Border {
//                radius: 4.0.into(),
//                ..Default::default()
//            },
//            ..Default::default()
//        },
//        Status::Hovered => Style {
//            background: Some(Background::Color(background_hover)),
//            text_color: foreground,
//            border: Border {
//                radius: 4.0.into(),
//                ..Default::default()
//            },
//            ..Default::default()
//        },
//        Status::Disabled => {
//            let active: Style = button(
//                foreground,
//                background,
//                background_hover,
//                Status::Active,
//            );
//
//            Style {
//                text_color: Color {
//                    a: 0.2,
//                    ..active.text_color
//                },
//                ..active
//            }
//        }
//    }
//}
//
