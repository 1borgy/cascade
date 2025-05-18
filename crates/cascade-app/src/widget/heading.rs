use iced::{font::Weight, widget::text, Font};

use crate::Element;

pub fn heading<'a, M>(label: impl ToString) -> Element<'a, M> {
    text(label.to_string())
        .size(32)
        .font(Font {
            weight: Weight::Bold,
            ..Default::default()
        })
        .into()
}
