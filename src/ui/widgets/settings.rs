use egui::{Align, Button, ComboBox, Context, Id, Key, Layout, Modifiers, ScrollArea, Ui};

use crate::ui::widgets::toggle::toggle;
use crate::{
    i18n::lang::{Language, TextBundle, get_i18n_text},
    model::config::Config,
    ui::widgets::modal::{ModalAction, ModalFrame},
};

#[derive(PartialEq, Clone, Copy, Hash)]
enum SettingsTab {
    General,
    Hotkeys,
}

// 用于在 egui 的内存中存储哪个快捷键正在被录制
#[derive(PartialEq, Clone, Copy, Hash)]
enum RecordingState {
    None,
    ShowScreenshot,
    CopyScreenshot,
}

pub fn render_settings_window(
    ctx: &Context,
    open: &mut bool,
    text: &TextBundle,
    config: &mut Config,
) -> ModalAction {
    let tab_id = Id::new("settings_tab_state");
    let mut current_tab = ctx
        .data(|d| d.get_temp(tab_id))
        .unwrap_or(SettingsTab::General);

    let recording_id = Id::new("hotkey_recording_state");
    let mut recording_state = ctx
        .data(|d| d.get_temp(recording_id))
        .unwrap_or(RecordingState::None);

    let mut action = ModalAction::None;

    ModalFrame::show(ctx, open, text.settings.title, |ui| {
        ui.set_min_width(700.0);
        ui.set_min_height(500.0);

        ui.vertical(|ui| {
            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), ui.available_height() - 40.0),
                    Layout::left_to_right(Align::Min),
                    |ui| {
                        render_sidebar(ui, &mut current_tab, text);
                        ui.separator();
                        ui.vertical(|ui| {
                            render_content_header(ui, current_tab, text);
                            ui.add_space(8.0);
                            ScrollArea::vertical().show(ui, |ui| {
                                render_content_body(
                                    ui,
                                    current_tab,
                                    config,
                                    text,
                                    &mut recording_state,
                                );
                            });
                        });
                    },
                );
            });

            ui.separator();
            ui.add_space(4.0);
            render_footer(ui, &mut action, text);
        });

        ctx.data_mut(|d| d.insert_temp(tab_id, current_tab));
        ctx.data_mut(|d| d.insert_temp(recording_id, recording_state));
        action
    });

    action
}

fn render_sidebar(ui: &mut Ui, current_tab: &mut SettingsTab, text: &TextBundle) {
    ui.vertical(|ui| {
        ui.set_width(180.0);
        ui.add_space(15.0);
        if ui
            .selectable_label(
                *current_tab == SettingsTab::General,
                format!("  {}", text.settings.general),
            )
            .clicked()
        {
            *current_tab = SettingsTab::General;
        }
        if ui
            .selectable_label(
                *current_tab == SettingsTab::Hotkeys,
                format!("  {}", text.settings.shortcut_key),
            )
            .clicked()
        {
            *current_tab = SettingsTab::Hotkeys;
        }
        ui.add_space(ui.available_height());
    });
}

fn render_content_header(ui: &mut Ui, current_tab: SettingsTab, text: &TextBundle) {
    ui.horizontal(|ui| {
        ui.add_space(5.0);
        let title = match current_tab {
            SettingsTab::General => &text.settings.general,
            SettingsTab::Hotkeys => &text.settings.shortcut_key,
        };
        ui.label(egui::RichText::new(format!("{} > {}", text.settings.title, title)).weak());
    });
}

fn render_content_body(
    ui: &mut Ui,
    current_tab: SettingsTab,
    config: &mut Config,
    text: &TextBundle,
    recording_state: &mut RecordingState,
) {
    ui.vertical(|ui| {
        ui.set_min_width(ui.available_width());
        match current_tab {
            SettingsTab::General => {
                ui.heading(text.settings.general);
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", text.settings.language));
                    let mut selected = config.language;
                    ComboBox::from_id_salt("lang_selector")
                        .selected_text(selected.as_str())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut selected, Language::Zh, Language::Zh.as_str());
                            ui.selectable_value(&mut selected, Language::En, Language::En.as_str());
                            ui.selectable_value(&mut selected, Language::Ja, Language::Ja.as_str());
                        });
                    if selected != config.language {
                        config.language = selected;
                    }
                });
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", text.settings.minimize_on_close));
                    ui.add(toggle(&mut config.minimize_on_close));
                });
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", text.settings.magnifier_enabled));
                    ui.add(toggle(&mut config.magnifier_enabled));
                });
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", text.settings.screenshot_hides_main_window));
                    ui.add(toggle(&mut config.screenshot_hides_main_window));
                });
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", text.settings.launch_on_startup));
                    ui.add(toggle(&mut config.launch_on_startup));
                });
            }
            SettingsTab::Hotkeys => {
                ui.heading(text.settings.shortcut_key);
                ui.add_space(10.0);

                egui::Grid::new("hotkeys_grid")
                    .num_columns(2)
                    .spacing([40.0, 10.0])
                    .striped(true)
                    .show(ui, |ui| {
                        render_hotkey_row(
                            ui,
                            text.shortcuts.screenshot,
                            &mut config.hotkeys.show_screenshot,
                            recording_state,
                            RecordingState::ShowScreenshot,
                        );
                        render_hotkey_row(
                            ui,
                            text.shortcuts.copy_color,
                            &mut config.hotkeys.copy_color,
                            recording_state,
                            RecordingState::CopyScreenshot,
                        );
                    });
            }
        }
    });
}

fn render_hotkey_row(
    ui: &mut Ui,
    label: &str,
    hotkey_str: &mut String,
    recording_state: &mut RecordingState,
    this_recorder: RecordingState,
) {
    ui.label(format!("{}:", label));

    let text = get_i18n_text(ui);
    let is_recording = *recording_state == this_recorder;
    let button_text = if is_recording {
        text.shortcuts.modified
    } else {
        hotkey_str.as_str()
    };

    if ui
        .add_sized([120.0, 20.0], Button::new(button_text))
        .clicked()
    {
        *recording_state = if is_recording {
            RecordingState::None
        } else {
            this_recorder
        };
    }

    if is_recording {
        ui.input(|i| {
            if i.key_pressed(Key::Escape) {
                *recording_state = RecordingState::None;
                return;
            }

            let modifiers = i.modifiers;

            for event in &i.events {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    ..
                } = event
                {
                    // Enter 键直接退出录制，不修改快捷键（避免与文本输入框冲突）
                    if *key == Key::Enter {
                        *recording_state = RecordingState::None;
                        break;
                    }
                    *hotkey_str = format_hotkey(modifiers, *key);
                    *recording_state = RecordingState::None;
                    break;
                }
            }
        });
    }
    ui.end_row();
}

/// 格式化热键字符串
fn format_hotkey(modifiers: Modifiers, key: Key) -> String {
    let mut parts = Vec::new();

    // 检查具体的修饰键状态
    if modifiers.ctrl {
        parts.push("Ctrl");
    }
    if modifiers.alt {
        parts.push("Alt");
    }
    if modifiers.shift {
        parts.push("Shift");
    }
    // 注意：mac_cmd 仅在 macOS 上对应 Command 键
    if modifiers.mac_cmd {
        parts.push("Cmd");
    }
    // 如果你在非 Mac 系统上想用 Win 键，可以使用 modifiers.command 或手动检查 os
    // 但通常 egui 的 modifiers.command 在 Mac 上是 Cmd，在 Win/Linux 上是 Ctrl

    // 获取按键名称
    // 使用 format!("{:?}") 可以获取枚举变体的名称，例如 Key::A -> "A"
    let key_name = format!("{:?}", key);
    parts.push(&key_name);

    parts.join("+")
}

fn render_footer(ui: &mut Ui, action: &mut ModalAction, text: &TextBundle) {
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        ui.add_space(10.0);
        if ui.button(text.settings.close).clicked() {
            *action = ModalAction::Close;
        }
        if ui.button(text.settings.apply).clicked() {
            *action = ModalAction::Apply;
        }
        ui.add_space(10.0);
    });
}
