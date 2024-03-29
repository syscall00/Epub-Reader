use druid::{
    widget::Flex, BoxConstraints, Color, Data, EventCtx, Point, RenderContext, Size, TextLayout,
    Widget, WidgetExt, WidgetPod,
};
use epub::doc::EpubDoc;

use crate::{
    core::{
        constants::commands::{InternalUICommand, INTERNAL_COMMAND},
        style,
    },
    data::home::Recent,
    widgets::RoundButton,
    PageType,
};
/**
 * Recent item widget. It display one recent item in the home page.
 * It contains the book cover, title, author, publisher and position in book.
 */
pub struct RecentWidget {
    title_label: druid::TextLayout<String>,
    creator_label: TextLayout<String>,
    publisher_label: TextLayout<String>,
    position_in_book_label: TextLayout<String>,

    image: WidgetPod<Recent, Box<dyn Widget<Recent>>>,
    remove_button: WidgetPod<Recent, Box<dyn Widget<Recent>>>,
}

impl RecentWidget {
    pub fn new() -> Self {
        let title_label = druid::TextLayout::default();
        let creator_label = druid::TextLayout::default();
        let publisher_label = druid::TextLayout::default();
        let position_in_book_label = druid::TextLayout::default();

        let image = WidgetPod::new(
            druid::widget::ViewSwitcher::new(
                |data: &Recent, _env| data.image_data.is_some(),
                |image, data, _env| match image {
                    true => druid::widget::Image::new(data.image_data.clone().unwrap()).boxed(),
                    false => Flex::column()
                        .with_child(
                            druid::widget::Label::new(String::from("Loading...")).padding(5.0),
                        )
                        .with_child(druid::widget::Spinner::new())
                        .boxed(),
                },
            )
            .boxed(),
        );

        let remove_button = WidgetPod::new(
            RoundButton::new(druid_material_icons::normal::navigation::CANCEL)
                .on_click(|ctx, data: &mut Recent, _env| {
                    ctx.submit_command(
                        INTERNAL_COMMAND.with(InternalUICommand::RemoveBook(data.path.clone())),
                    );
                })
                .padding(5.0)
                .boxed(),
        );
        RecentWidget {
            title_label,
            creator_label,
            publisher_label,
            position_in_book_label,
            image,
            remove_button,
        }
    }
}

impl Widget<Recent> for RecentWidget {
    fn event(
        &mut self,
        ctx: &mut EventCtx,
        event: &druid::Event,
        data: &mut Recent,
        _env: &druid::Env,
    ) {
        match event {
            druid::Event::MouseUp(mouse_event) => {
                // if first half of the widget is clicked, open the book
                if mouse_event.pos.x < ctx.size().width / 2.0 {
                    ctx.set_handled();

                    ctx.submit_command(druid::Command::new(
                        INTERNAL_COMMAND,
                        InternalUICommand::OpenRecent(data.clone()),
                        druid::Target::Auto,
                    ));
                    ctx.submit_command(
                        INTERNAL_COMMAND.with(InternalUICommand::UINavigate(PageType::Reader)),
                    );
                }
            }
            druid::Event::MouseMove(mouse_event) => {
                let pointer = if mouse_event.pos.x < ctx.size().width / 2.0 {
                    druid::Cursor::Pointer
                } else {
                    druid::Cursor::Arrow
                };
                ctx.set_cursor(&pointer);
                ctx.request_paint();
            }

            druid::Event::Command(cmd) => {
                if cmd.is(INTERNAL_COMMAND) {
                    match cmd.get_unchecked(INTERNAL_COMMAND) {
                        InternalUICommand::HomePageImageLoaded(img_data) => {
                            data.image_data = Some(img_data.to_owned());
                            ctx.request_paint();
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        self.image.event(ctx, event, data, _env);
        self.remove_button.event(ctx, event, data, _env);
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &Recent,
        _env: &druid::Env,
    ) {
        match event {
            druid::LifeCycle::WidgetAdded => {
                let mut ep = EpubDoc::new(data.path.clone()).unwrap();
                const UNTITLED_BOOK: &str = "Untitled";
                const UNKNOWN_CREATOR_OR_PUBLISHER: &str = "Unknown";
                let binding = ep.get_cover();

                if binding.is_ok() {
                    if let Ok(img_data) = binding.as_ref() {
                        // retrieve widget id
                        render_image_slowly(
                            ctx.get_external_handle(),
                            img_data.to_owned(),
                            ctx.widget_id(),
                        );
                    }
                }

                let title = ep
                    .mdata("title")
                    .unwrap_or(UNTITLED_BOOK.to_string())
                    .to_string();
                let creator = ep
                    .mdata("creator")
                    .unwrap_or(UNKNOWN_CREATOR_OR_PUBLISHER.to_string())
                    .to_string();
                let publisher = ep
                    .mdata("publisher")
                    .unwrap_or(UNKNOWN_CREATOR_OR_PUBLISHER.to_string())
                    .to_string();

                self.title_label.set_text(title);
                self.title_label.set_text_size(18.);
                self.title_label.set_text_color(Color::WHITE);

                self.creator_label.set_text(creator);
                self.creator_label.set_text_size(14.);
                self.creator_label.set_text_color(Color::WHITE);

                self.publisher_label.set_text(publisher);
                self.publisher_label.set_text_size(14.);
                self.publisher_label.set_text_color(Color::WHITE);

                self.position_in_book_label
                    .set_text(if let Some(pos) = &data.reached_position {
                        pos.to_string()
                    } else {
                        "Never opened".to_string()
                    });
                self.position_in_book_label.set_text_size(14.);
                self.position_in_book_label.set_text_color(Color::WHITE);
            }
            _ => {}
        }
        self.image.lifecycle(ctx, event, data, _env);
        self.remove_button.lifecycle(ctx, event, data, _env);
    }

    fn update(
        &mut self,
        ctx: &mut druid::UpdateCtx,
        old_data: &Recent,
        data: &Recent,
        env: &druid::Env,
    ) {
        use downcast_rs::Downcast;

        if !old_data.same(data) {
            if !data.image_data.same(&old_data.image_data) {
                self.image.update(ctx, data, env);
                // get image and dereference it as druid::widget::Image
                if let Some(image_data) = &data.image_data {
                    let image = self
                        .image
                        .widget_mut()
                        .as_any_mut()
                        .downcast_mut::<druid::widget::Image>();
                    if let Some(image_dataa) = image {
                        image_dataa.set_image_data(image_data.clone());
                    }
                }
            }

            ctx.request_layout();
            ctx.request_layout();
            ctx.request_paint();
        }
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &Recent,
        env: &druid::Env,
    ) -> druid::Size {
        const IMAGE_HEIGHT: f64 = 180.;
        const TITLE_TEXT_WRAP: f64 = 150.;

        self.title_label.set_wrap_width(TITLE_TEXT_WRAP);
        self.title_label.layout();
        self.title_label.rebuild_if_needed(ctx.text(), env);

        self.creator_label.set_wrap_width(TITLE_TEXT_WRAP);
        self.creator_label.layout();
        self.creator_label.rebuild_if_needed(ctx.text(), env);

        self.publisher_label.set_wrap_width(TITLE_TEXT_WRAP);
        self.publisher_label.layout();
        self.publisher_label.rebuild_if_needed(ctx.text(), env);

        self.position_in_book_label.set_wrap_width(TITLE_TEXT_WRAP);
        self.position_in_book_label.layout();
        self.position_in_book_label
            .rebuild_if_needed(ctx.text(), env);

        self.image.layout(
            ctx,
            &BoxConstraints::tight(Size::new(130., IMAGE_HEIGHT)),
            data,
            env,
        );
        self.image.set_origin(ctx, data, env, Point::new(10., 10.));

        let btn_size = self.remove_button.layout(ctx, bc, data, env);
        // put remove button on the right side of the widget
        self.remove_button.set_origin(
            ctx,
            data,
            env,
            Point::new(bc.max().width - btn_size.width, 100. - btn_size.height / 2.),
        );
        druid::Size::new(bc.max().width, 200.)
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &Recent, _env: &druid::Env) {
        let size = ctx.size();
        ctx.fill(
            size.to_rect(),
            &style::get_color_unchecked(style::PRIMARY_DARK),
        );
        const LABEL_PADDING: f64 = 5.;

        let mut y = 15.;
        self.title_label.draw(ctx, Point::new(150., y));
        y += self.title_label.size().height + LABEL_PADDING;
        self.creator_label.draw(ctx, Point::new(150., y));
        y += self.creator_label.size().height + LABEL_PADDING;

        self.publisher_label.draw(ctx, Point::new(150., y));
        y += self.publisher_label.size().height + LABEL_PADDING;

        self.position_in_book_label.draw(ctx, Point::new(150., y));
        self.image.paint(ctx, data, _env);

        if ctx.is_hot() {
            self.remove_button.paint(ctx, data, _env);
        }
    }
}
// main page and contains list view of contacts
// notice that this must return Box<dyn Widget<YourState>> instead of impl Widget<YourState>
// navigator needs Boxed widgets in order to store the widgets

fn render_image_slowly(
    sink: druid::ExtEventSink,
    img_data: Vec<u8>,
    widget_target: druid::WidgetId,
) {
    std::thread::spawn(move || {
        if let Ok(img_buf) = druid::ImageBuf::from_data(&img_data) {
            sink.submit_command(
                INTERNAL_COMMAND,
                InternalUICommand::HomePageImageLoaded(img_buf),
                druid::Target::Widget(widget_target),
            )
            .expect("command failed to submit");
        }
    });
}
