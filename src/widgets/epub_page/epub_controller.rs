use druid::{
    widget::{Controller, Flex},
    Env, Event, EventCtx, LensExt, Widget, WidgetExt,
};

use crate::{
    core::constants::commands::{InternalUICommand, INTERNAL_COMMAND},
    data::{epub::EpubData, AppState, PagePosition},
    widgets::EditWidget,
    PageType,
};

pub struct EpubPageController;
/**
 * Controller for the epub page
 * It mainly handles the opening and closing of the popups or
 * the navigation to home page
 */
impl Controller<AppState, Flex<AppState>> for EpubPageController {
    fn event(
        &mut self,
        child: &mut Flex<AppState>,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppState,
        env: &Env,
    ) {
        match event {
            Event::Command(cmd) => {
                if let Some(cmd) = cmd.get(INTERNAL_COMMAND) {
                   match cmd {
                        // Go back to home page
                        InternalUICommand::GoToMenu => {
                            ctx.submit_command(INTERNAL_COMMAND.with(
                                InternalUICommand::UpdateBookInfo(data.epub_data.get_epub_path()),
                            ));
                            ctx.submit_command(
                                INTERNAL_COMMAND
                                    .with(InternalUICommand::UINavigate(PageType::Home)),
                            );

                            ctx.set_handled();
                        }
                        // Create OCR popup
                        InternalUICommand::OpenOCRDialog => {
                            ctx.new_sub_window(
                                druid::WindowConfig::default()
                                    .show_titlebar(true)
                                    .set_level(druid::WindowLevel::AppWindow),
                                crate::widgets::build_ocr_ui().lens(EpubData::ocr_data),
                                data.epub_data.clone(),
                                env.clone(),
                            );
                            ctx.set_handled();
                        }
                        InternalUICommand::RequestOCRSearch(image_path) => {
                            let strings = data.epub_data.get_only_strings();

                            start_ocr_search_in_thread(
                                ctx.get_external_handle(),
                                image_path.to_owned(),
                                strings.to_owned(),
                            );
                            ctx.request_update();
                            ctx.set_handled();
                        }
                        InternalUICommand::RequestReverseOCR((img1, img2)) => {
                            let strings = data.epub_data.get_only_strings();
                            start_reverse_ocr_search_in_thread(
                                ctx.get_external_handle(),
                                img1.to_owned(),
                                img2.to_owned(),
                                strings.to_owned(),
                                data.epub_data.page_position.clone(),
                            );
                            ctx.request_update();
                            ctx.set_handled();
                        }
                        // Create Edit popup
                        InternalUICommand::OpenEditDialog => {
                            if !data.epub_data.edit_data.is_editing() {
                                data.epub_data.edit_data.set_editing(true);
                                let window_config = druid::WindowConfig::default();

                                ctx.new_sub_window(
                                    window_config,
                                    EditWidget::new()
                                        .lens(AppState::epub_data.then(EpubData::edit_data)),
                                    data.clone(),
                                    env.clone(),
                                );
                                ctx.set_handled();
                            }
                        }
                        InternalUICommand::SaveModification(path) => {
                            data.epub_data.save_new_epub(path);
                            ctx.request_update();
                            ctx.set_handled();
                        }

                        _ => {}
                    }
                }
            }
            _ => {}
        }
        child.event(ctx, event, data, env);
    }
}

fn start_ocr_search_in_thread(
    sink: druid::ExtEventSink,
    image_path: String,
    strings: Vec<Vec<String>>,
) {
    std::thread::spawn(move || {
        let res = crate::ocr::search_with_ocr_input(strings, &image_path);
        //match res {
            //Ok(position) => {
                sink.submit_command(
                    INTERNAL_COMMAND,
                    InternalUICommand::OCRSearchCompleted(res),
                    druid::Target::Global,
                )
                .expect("command failed to submit");
            //}
            //Err(e) => {
            //    println!("Error: ");
            //}
       // }
        
    });
}

fn start_reverse_ocr_search_in_thread(
    sink: druid::ExtEventSink,
    image_1: String,
    image_2: String,
    strings: Vec<Vec<String>>,
    current_position: PagePosition,
) {
    std::thread::spawn(move || {
        let res = crate::ocr::reverse_search_with_ocr_input(
            strings,
            &image_1,
            &image_2,
            &current_position,
        );
        sink.submit_command(
            INTERNAL_COMMAND,
            InternalUICommand::ReverseOCRCompleted(res),
            druid::Target::Global,
        )
        .expect("command failed to submit");
    });
}
