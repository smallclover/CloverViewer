use egui::{Align, ComboBox, Context, Layout, Id, Ui, ScrollArea};
use crate::i18n::{get_text, Language, TextBundle};
use crate::ui::modal::{ModalAction, ModalFrame};

#[derive(PartialEq, Clone, Copy)]
enum SettingsTab {
    General,
    Appearance,
    Advanced,
}

pub fn render_settings_window(
    ctx: &Context,
    open: &mut bool,
    render_lang: Language,
    lang_setting: &mut Language,
) -> ModalAction {
    let text = get_text(render_lang);
    let tab_id = Id::new("settings_tab_state");
    let mut current_tab = ctx.data(|d| d.get_temp(tab_id)).unwrap_or(SettingsTab::General);
    let mut action = ModalAction::None;

    ModalFrame::show(ctx, open, text.settings_title, |ui| {
        ui.set_min_width(700.0);
        ui.set_min_height(500.0);

        ui.vertical(|ui| {
            // 1. 上部核心区域：左右分栏
            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), ui.available_height() - 40.0), // 为底部按钮留出空间
                    Layout::left_to_right(Align::Min),
                    |ui| {
                        // --- 左侧：侧边栏 ---
                        render_sidebar(ui, &mut current_tab, &text);
                        // 分割线
                        ui.separator();
                        // --- 右侧：内容区 ---
                        ui.vertical(|ui| {
                            render_content_header(ui, current_tab, &text);
                            ui.add_space(8.0);

                            ScrollArea::vertical().show(ui, |ui| {
                                render_content_body(ui, current_tab, lang_setting, &text);
                            });
                        });
                    },
                );
            });

            // 2. 底部：操作按钮
            ui.separator();
            ui.add_space(4.0);
            render_footer(ui, &mut action, &text);
        });

        // 保存状态
        ctx.data_mut(|d| d.insert_temp(tab_id, current_tab));
        action
    });

    action
}

/// 渲染左侧导航栏
fn render_sidebar(ui: &mut Ui, current_tab: &mut SettingsTab, text: &TextBundle) {
    ui.vertical(|ui| {
        ui.set_width(180.0); // 固定侧边栏宽度
        ui.add_space(15.0);

        // 导航列表
        if ui.selectable_label(*current_tab == SettingsTab::General, format!("  {}", text.settings_general)).clicked() {
            *current_tab = SettingsTab::General;
        }
        if ui.selectable_label(*current_tab == SettingsTab::Appearance, format!("  {}", text.settings_appearance)).clicked() {
            *current_tab = SettingsTab::Appearance;
        }
        if ui.selectable_label(*current_tab == SettingsTab::Advanced, format!("  {}", text.settings_advanced)).clicked() {
            *current_tab = SettingsTab::Advanced;
        }

        ui.add_space(ui.available_height()); // 撑开底部
    });
}

/// 渲染右侧顶部的面包屑/标题
fn render_content_header(ui: &mut Ui, current_tab: SettingsTab, text: &TextBundle) {
    ui.horizontal(|ui| {
        ui.add_space(5.0);
        let title = match current_tab {
            SettingsTab::General => text.settings_general,
            SettingsTab::Appearance => text.settings_appearance,
            SettingsTab::Advanced => text.settings_advanced,
        };
        // 仿照图片中的 Rust > 外部 Linter 路径显示
        ui.label(egui::RichText::new(format!("设置 > {}", title)).weak());
    });
}

/// 渲染具体的设置表单项
fn render_content_body(ui: &mut Ui, current_tab: SettingsTab, lang_setting: &mut Language, text: &TextBundle) {
    ui.vertical(|ui| {
        ui.set_min_width(ui.available_width());

        match current_tab {
            SettingsTab::General => {
                ui.heading(text.settings_general);
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label(format!("{}:", text.settings_language));
                    let mut selected = *lang_setting;
                    ComboBox::from_id_salt("lang_selector")
                        .selected_text(selected.as_str())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut selected, Language::Zh, Language::Zh.as_str());
                            ui.selectable_value(&mut selected, Language::En, Language::En.as_str());
                            ui.selectable_value(&mut selected, Language::Ja, Language::Ja.as_str());
                        });
                    if selected != *lang_setting { *lang_setting = selected; }
                });
            }
            SettingsTab::Appearance => {
                ui.heading(text.settings_appearance);
                // 具体的 UI 细节可以在此处继续细分函数
            }
            SettingsTab::Advanced => {
                ui.heading(text.settings_advanced);
            }
        }
    });
}

/// 渲染底部按钮区域
fn render_footer(ui: &mut Ui, action: &mut ModalAction, text: &TextBundle) {
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        ui.add_space(10.0); // 右边距
        if ui.button(text.settings_close).clicked() {
            *action = ModalAction::Close;
        }
        if ui.button(text.settings_apply).clicked() {
            *action = ModalAction::Apply;
        }
        ui.add_space(10.0);
    });
}