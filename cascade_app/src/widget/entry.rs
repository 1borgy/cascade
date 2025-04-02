use cascade::save;
use iced::{
    widget::{button, checkbox, text},
    Length,
};

use crate::{theme, Element, Row};

pub fn selectable<'a, Message>(
    entry: &save::Entry,
    selected: bool,
    on_press_maybe: Option<Message>,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    let checkbox = checkbox("", selected)
        .style(match selected {
            true => theme::checkbox::entry_selected,
            false => theme::checkbox::entry_unselected,
        })
        .on_toggle_maybe(
            on_press_maybe
                .clone()
                .map(|on_press| move |_| on_press.clone()),
        );

    let name = entry.name.clone();
    button(Row::new().push(checkbox).push(text(name)))
        .style(match selected {
            true => theme::button::entry_selected,
            false => theme::button::entry_unselected,
        })
        .on_press_maybe(on_press_maybe)
        .width(Length::Fill)
        .into()
}

// pub fn queued<'a, Message>(
//     name: String,
//     on_press_maybe: Option<Message>,
// ) -> Element<'a, Message>
// where
//     Message: 'a + Clone,
// {
//     button(text(name))
//         .style(theme::button::entry_queued)
//         .on_press_maybe(on_press_maybe)
//         .width(Length::Fill)
//         .into()
// }
