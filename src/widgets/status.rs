use cosmic::{
    iced::mouse::Interaction,
    iced_widget::scrollable::{Direction, Scrollbar},
    widget, Element, Task,
};
use mastodon_async::prelude::{Account, Status, StatusId};

use crate::{app, utils::Cache};

#[derive(Debug, Clone)]
pub enum Message {
    OpenAccount(Account),
    ExpandStatus(StatusId),
    Reply(StatusId),
    Favorite(StatusId, bool),
    Boost(StatusId),
    Bookmark(StatusId),
    OpenLink(String),
}

pub fn status<'a>(status: &'a Status, cache: &'a Cache) -> Element<'a, Message> {
    let spacing = cosmic::theme::active().cosmic().spacing;

    let status = if let Some(reblog) = &status.reblog {
        let reblog = cache.statuses.get(&reblog.id.to_string()).unwrap_or(reblog);

        let indicator = widget::button::custom(
            widget::row()
                .push(
                    cache
                        .handles
                        .get(&status.account.avatar)
                        .map(|avatar| widget::image(avatar).width(20).height(20))
                        .unwrap_or(crate::utils::fallback_avatar().width(20).height(20)),
                )
                .push(widget::text(format!(
                    "{} boosted",
                    status.account.display_name
                )))
                .spacing(spacing.space_xs),
        )
        .on_press(Message::OpenAccount(reblog.account.clone()));

        widget::column()
            .push(indicator)
            .push(self::status(&*reblog, cache))
            .spacing(spacing.space_xs)
    } else {
        let display_name = format!(
            "{} @{}",
            status.account.display_name, status.account.username
        );

        let content = widget::row()
            .push(
                widget::button::image(
                    cache
                        .handles
                        .get(&status.account.avatar)
                        .cloned()
                        .unwrap_or(crate::utils::fallback_handle()),
                )
                .width(50)
                .height(50)
                .on_press(Message::OpenAccount(status.account.clone())),
            )
            .push(
                widget::column()
                    .push(
                        widget::button::link(display_name)
                            .on_press(Message::OpenAccount(status.account.clone())),
                    )
                    .push(
                        widget::MouseArea::new(widget::text(
                            html2text::config::rich()
                                .string_from_read(status.content.as_bytes(), 700)
                                .unwrap(),
                        ))
                        .interaction(Interaction::Pointer)
                        .on_press(Message::ExpandStatus(status.id.clone())),
                    )
                    .spacing(spacing.space_xxs),
            )
            .spacing(spacing.space_xs);

        let tags: Option<Element<_>> = (!status.tags.is_empty()).then(|| {
            widget::row()
                .spacing(spacing.space_xxs)
                .extend(
                    status
                        .tags
                        .iter()
                        .map(|tag| {
                            widget::button::suggested(format!("#{}", tag.name.clone()))
                                .on_press(Message::OpenLink(tag.url.clone()))
                                .into()
                        })
                        .collect::<Vec<Element<Message>>>(),
                )
                .into()
        });

        let attachments = status
            .media_attachments
            .iter()
            .filter_map(|media| {
                cache
                    .handles
                    .get(&media.preview_url.to_string())
                    .map(|handle| {
                        widget::button::image(handle.clone())
                            .on_press_maybe(media.url.as_ref().cloned().map(Message::OpenLink))
                            .into()
                    })
            })
            .collect::<Vec<Element<Message>>>();

        let media = (!status.media_attachments.is_empty()).then_some({
            widget::scrollable(widget::row().extend(attachments).spacing(spacing.space_xxs))
                .direction(Direction::Horizontal(Scrollbar::new()))
        });

        let actions = widget::row()
            .push(
                widget::button::icon(widget::icon::from_name("mail-replied-symbolic"))
                    .label(status.replies_count.unwrap_or_default().to_string())
                    .on_press(Message::Reply(status.id.clone())),
            )
            .push(
                widget::button::icon(widget::icon::from_name("emblem-shared-symbolic"))
                    .label(status.reblogs_count.to_string())
                    .on_press(Message::Boost(status.id.clone())),
            )
            .push(
                widget::button::icon(widget::icon::from_name("starred-symbolic"))
                    .label(status.favourites_count.to_string())
                    .class(if status.favourited.unwrap() {
                        cosmic::theme::Button::Link
                    } else {
                        cosmic::theme::Button::Icon
                    })
                    .on_press(Message::Favorite(
                        status.id.clone(),
                        status.favourited.unwrap(),
                    )),
            )
            .push(
                widget::button::icon(widget::icon::from_name("bookmark-new-symbolic"))
                    .on_press(Message::Bookmark(status.id.clone())),
            )
            .padding(spacing.space_xs)
            .spacing(spacing.space_xs);

        widget::column()
            .push(content)
            .push_maybe(media)
            .push_maybe(tags)
            .push(actions)
            .spacing(spacing.space_xs)
    };

    widget::settings::flex_item_row(vec![status.into()])
        .padding(spacing.space_xs)
        .into()
}

pub fn update(message: Message) -> Task<app::Message> {
    match message {
        Message::OpenAccount(account) => cosmic::task::message(app::Message::ToggleContextPage(
            app::ContextPage::Account(account),
        )),
        Message::ExpandStatus(id) => cosmic::task::message(app::Message::ToggleContextPage(
            app::ContextPage::Status(id),
        )),
        Message::Reply(status_id) => cosmic::task::message(cosmic::app::message::none()),
        Message::Favorite(status_id, favorited) => cosmic::task::message(app::Message::Status(
            Message::Favorite(status_id, favorited),
        )),
        Message::Boost(status_id) => cosmic::task::message(cosmic::app::message::none()),
        Message::Bookmark(status_id) => cosmic::task::message(cosmic::app::message::none()),
        Message::OpenLink(url) => cosmic::task::message(app::Message::Open(url)),
    }
}
