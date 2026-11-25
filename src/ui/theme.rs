//! Tema visual minimalista e moderno para o Orange PC Monitor HUD.

use eframe::egui::{self, Color32, Rounding, Stroke, Visuals};

pub struct Theme;

impl Theme {
    // Cores de fundo em formato premultiplied válido (r, g, b <= a)
    pub const BG_WINDOW: Color32 = Color32::from_rgba_premultiplied(13, 16, 22, 242);
    pub const BG_CARD: Color32 = Color32::from_rgba_premultiplied(22, 27, 36, 242);
    pub const BG_CARD_HOVER: Color32 = Color32::from_rgba_premultiplied(30, 36, 48, 245);
    pub const BG_PROGRESS_BAR: Color32 = Color32::from_rgba_premultiplied(36, 42, 54, 230);
    pub const BG_CHART: Color32 = Color32::from_rgba_premultiplied(12, 15, 21, 220);

    // Bordas sutis
    pub const BORDER_COLOR: Color32 = Color32::from_rgba_premultiplied(115, 62, 12, 120);
    pub const CARD_BORDER_COLOR: Color32 = Color32::from_rgba_premultiplied(44, 52, 68, 190);
    pub const BORDER_STROKE: Stroke = Stroke {
        width: 1.0,
        color: Self::BORDER_COLOR,
    };
    pub const CARD_BORDER_STROKE: Stroke = Stroke {
        width: 1.0,
        color: Self::CARD_BORDER_COLOR,
    };

    // Cores de destaque (Branding Orange)
    pub const ORANGE_ACCENT: Color32 = Color32::from_rgb(255, 126, 26);
    pub const ORANGE_HOVER: Color32 = Color32::from_rgb(255, 152, 72);
    pub const ORANGE_DIM: Color32 = Color32::from_rgba_premultiplied(70, 35, 10, 225);

    // Cores de botões do cabeçalho
    pub const BTN_IDLE: Color32 = Color32::from_rgba_premultiplied(28, 34, 46, 210);
    pub const BTN_HOVER: Color32 = Color32::from_rgba_premultiplied(42, 50, 66, 235);
    pub const BTN_CLOSE_HOVER: Color32 = Color32::from_rgba_premultiplied(185, 45, 45, 235);

    // Cores de texto
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(242, 245, 249);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(154, 165, 180);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(106, 118, 134);

    // Cores de status de telemetria
    pub const STATUS_EXCELLENT: Color32 = Color32::from_rgb(34, 197, 94); // Emerald 500
    pub const STATUS_NORMAL: Color32 = Color32::from_rgb(56, 189, 248); // Sky 400
    pub const STATUS_WARNING: Color32 = Color32::from_rgb(245, 158, 11); // Amber 500
    pub const STATUS_CRITICAL: Color32 = Color32::from_rgb(239, 68, 68); // Red 500

    /// Aplica o tema escuro customizado do Orange HUD no contexto do egui.
    pub fn apply(ctx: &egui::Context) {
        let mut visuals = Visuals::dark();
        visuals.window_fill = Self::BG_WINDOW;
        visuals.panel_fill = Color32::TRANSPARENT;
        visuals.faint_bg_color = Self::BG_CARD;
        visuals.extreme_bg_color = Self::BG_CHART;
        visuals.override_text_color = Some(Self::TEXT_PRIMARY);

        visuals.widgets.noninteractive.bg_fill = Self::BG_CARD;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, Self::CARD_BORDER_COLOR);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, Self::TEXT_SECONDARY);
        visuals.widgets.noninteractive.rounding = Rounding::same(6.0);

        visuals.widgets.inactive.bg_fill = Self::BTN_IDLE;
        visuals.widgets.inactive.weak_bg_fill = Self::BTN_IDLE;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Self::CARD_BORDER_COLOR);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, Self::TEXT_SECONDARY);
        visuals.widgets.inactive.rounding = Rounding::same(5.0);

        visuals.widgets.hovered.bg_fill = Self::BTN_HOVER;
        visuals.widgets.hovered.weak_bg_fill = Self::BTN_HOVER;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, Self::ORANGE_ACCENT);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, Self::TEXT_PRIMARY);
        visuals.widgets.hovered.rounding = Rounding::same(5.0);

        visuals.widgets.active.bg_fill = Self::ORANGE_DIM;
        visuals.widgets.active.weak_bg_fill = Self::ORANGE_DIM;
        visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, Self::ORANGE_ACCENT);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Self::TEXT_PRIMARY);
        visuals.widgets.active.rounding = Rounding::same(5.0);

        visuals.selection.bg_fill = Self::ORANGE_DIM;
        visuals.selection.stroke = Stroke::new(1.0_f32, Self::ORANGE_ACCENT);

        ctx.set_visuals(visuals);
    }

    /// Retorna cor semântica com base na porcentagem de uso (0.0 - 100.0).
    pub fn usage_color(percent: f32) -> Color32 {
        if percent < 60.0 {
            Self::STATUS_EXCELLENT
        } else if percent < 85.0 {
            Self::STATUS_WARNING
        } else {
            Self::STATUS_CRITICAL
        }
    }

    /// Retorna cor semântica para temperatura (°C).
    pub fn temp_color(temp_c: f32) -> Color32 {
        if temp_c < 65.0 {
            Self::STATUS_EXCELLENT
        } else if temp_c < 80.0 {
            Self::STATUS_WARNING
        } else {
            Self::STATUS_CRITICAL
        }
    }
}
