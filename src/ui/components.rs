//! Componentes visuais ultra leves e modernos para o HUD minimalista.

use super::theme::Theme;
use eframe::egui::{self, Color32, Frame, Margin, Pos2, Rect, Response, Rounding, Sense, Stroke, Ui, Vec2};

/// Desenha uma barra de progresso horizontal com cantos arredondados.
pub fn mini_progress_bar(ui: &mut Ui, percent: f32, color: Color32, height: f32) {
    let desired_size = Vec2::new(ui.available_width().max(40.0), height);
    let (rect, _response) = ui.allocate_exact_size(desired_size, Sense::hover());

    let painter = ui.painter();
    let radius = height * 0.5;
    painter.rect_filled(rect, radius, Theme::BG_PROGRESS_BAR);

    let fill_width = (rect.width() * (percent.clamp(0.0, 100.0) / 100.0)).max(0.0);
    if fill_width > 0.5 {
        let fill_rect = Rect::from_min_size(rect.min, Vec2::new(fill_width, rect.height()));
        painter.rect_filled(fill_rect, radius, color);
    }
}

/// Container de seção estilo glass card para agrupar métricas.
pub fn section_card(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    Frame::none()
        .fill(Theme::BG_CARD)
        .rounding(Rounding::same(8.0))
        .stroke(Theme::CARD_BORDER_STROKE)
        .inner_margin(Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add_contents(ui);
        });
}

/// Cartão compacto de métrica com estilo glass, indicador colorido, valor e barra de progresso.
pub fn metric_tile(
    ui: &mut Ui,
    title: &str,
    value_text: &str,
    sub_text: &str,
    percent: f32,
    color: Color32,
    card_width: f32,
) {
    Frame::none()
        .fill(Theme::BG_CARD)
        .rounding(Rounding::same(8.0))
        .stroke(Theme::CARD_BORDER_STROKE)
        .inner_margin(Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.set_width((card_width - 20.0).max(80.0));
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    // Indicador circular de cor da métrica
                    let (dot_rect, _) = ui.allocate_exact_size(Vec2::new(6.0, 10.0), Sense::hover());
                    ui.painter()
                        .circle_filled(dot_rect.center(), 3.0, color);

                    ui.label(
                        egui::RichText::new(title)
                            .size(10.0)
                            .color(Theme::TEXT_SECONDARY)
                            .strong(),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(sub_text)
                                .size(9.5)
                                .color(Theme::TEXT_MUTED),
                        );
                    });
                });

                ui.add_space(3.0);

                ui.label(
                    egui::RichText::new(value_text)
                        .size(18.0)
                        .color(Theme::TEXT_PRIMARY)
                        .strong(),
                );

                ui.add_space(5.0);
                mini_progress_bar(ui, percent, color, 4.5);
            });
        });
}

/// Desenha um gráfico sparkline em tempo real com grade sutil e curva destacada.
pub fn sparkline(ui: &mut Ui, samples: &[f32], max_val: f32, height: f32, stroke_color: Color32) {
    let desired_size = Vec2::new(ui.available_width().max(80.0), height);
    let (rect, _response) = ui.allocate_exact_size(desired_size, Sense::hover());
    let painter = ui.painter();

    // Fundo do gráfico com borda discreta
    painter.rect_filled(rect, 5.0, Theme::BG_CHART);
    painter.rect_stroke(rect, 5.0, Theme::CARD_BORDER_STROKE);

    // Linha guia horizontal central (50%)
    let mid_y = rect.center().y;
    painter.line_segment(
        [Pos2::new(rect.min.x + 2.0, mid_y), Pos2::new(rect.max.x - 2.0, mid_y)],
        Stroke::new(0.6_f32, Color32::from_rgba_premultiplied(50, 58, 72, 110)),
    );

    if samples.len() < 2 {
        return;
    }

    let n = samples.len();
    let pad_x = 4.0;
    let pad_y = 4.0;
    let inner_w = (rect.width() - pad_x * 2.0).max(1.0);
    let inner_h = (rect.height() - pad_y * 2.0).max(1.0);
    let step_x = inner_w / (n - 1) as f32;
    let safe_max = if max_val <= 0.0 { 100.0 } else { max_val };

    let mut points = Vec::with_capacity(n);
    for (i, &val) in samples.iter().enumerate() {
        let x = rect.min.x + pad_x + i as f32 * step_x;
        let norm_y = (val / safe_max).clamp(0.0, 1.0);
        let y = rect.max.y - pad_y - (norm_y * inner_h);
        points.push(Pos2::new(x, y));
    }

    // Linha do gráfico
    for i in 0..points.len() - 1 {
        painter.line_segment(
            [points[i], points[i + 1]],
            Stroke::new(1.6_f32, stroke_color),
        );
    }

    // Destaque no ponto mais recente
    if let Some(&last) = points.last() {
        painter.circle_filled(last, 2.5, stroke_color);
    }
}

/// Linha de estatística Chave - Valor.
pub fn stat_row(ui: &mut Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(label)
                .size(10.5)
                .color(Theme::TEXT_SECONDARY),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(value)
                    .size(10.5)
                    .color(Theme::TEXT_PRIMARY)
                    .strong(),
            );
        });
    });
}

/// Badge de status arredondado.
pub fn badge(ui: &mut Ui, text: &str, bg_color: Color32, text_color: Color32) -> Response {
    let padding = Vec2::new(6.0, 2.0);
    let galley = ui.painter().layout_no_wrap(
        text.to_string(),
        egui::FontId::proportional(9.5),
        text_color,
    );
    let size = galley.size() + padding * 2.0;
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        painter.rect_filled(rect, 4.0, bg_color);
        painter.galley(rect.min + padding, galley, text_color);
    }

    response
}

/// Botão compacto em formato pílula para controles do cabeçalho.
pub fn header_pill_button(
    ui: &mut Ui,
    label: &str,
    active: bool,
    is_close: bool,
) -> Response {
    let padding = Vec2::new(6.0, 3.0);
    let text_color = if active {
        Theme::ORANGE_ACCENT
    } else {
        Theme::TEXT_SECONDARY
    };
    let galley = ui.painter().layout_no_wrap(
        label.to_string(),
        egui::FontId::proportional(9.5),
        text_color,
    );
    let size = Vec2::new((galley.size().x + padding.x * 2.0).max(20.0), 19.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let bg = if response.hovered() {
            if is_close {
                Theme::BTN_CLOSE_HOVER
            } else {
                Theme::BTN_HOVER
            }
        } else if active {
            Theme::ORANGE_DIM
        } else {
            Theme::BTN_IDLE
        };

        let border = if active {
            Stroke::new(1.0_f32, Theme::ORANGE_ACCENT)
        } else if response.hovered() && !is_close {
            Stroke::new(1.0_f32, Theme::ORANGE_HOVER)
        } else {
            Stroke::new(0.8_f32, Theme::CARD_BORDER_COLOR)
        };

        let final_text_col = if response.hovered() {
            Theme::TEXT_PRIMARY
        } else {
            text_color
        };

        painter.rect_filled(rect, 5.0, bg);
        painter.rect_stroke(rect, 5.0, border);

        let centered_galley = ui.painter().layout_no_wrap(
            label.to_string(),
            egui::FontId::proportional(9.5),
            final_text_col,
        );
        let text_pos = Pos2::new(
            rect.center().x - centered_galley.size().x * 0.5,
            rect.center().y - centered_galley.size().y * 0.5,
        );
        painter.galley(text_pos, centered_galley, final_text_col);
    }

    response
}
