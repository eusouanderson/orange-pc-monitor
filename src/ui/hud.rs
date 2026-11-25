//! Janela flutuante desktop minimalista (HUD) usando eframe/egui.

use super::components::{badge, header_pill_button, metric_tile, section_card, sparkline, stat_row};
use super::theme::Theme;
use crate::config::AppConfig;
use crate::core::engine::TelemetryEngine;
use crate::core::snapshot::{ActiveAlert, AlertSeverity, SystemSnapshot};
use eframe::egui::{self, Color32, Frame, Margin, Rounding, Vec2};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HudViewMode {
    Compact,
    Expanded,
    Diagnostic,
}

impl HudViewMode {
    pub fn window_size(self) -> Vec2 {
        match self {
            HudViewMode::Compact => Vec2::new(360.0, 232.0),
            HudViewMode::Expanded => Vec2::new(410.0, 540.0),
            HudViewMode::Diagnostic => Vec2::new(460.0, 600.0),
        }
    }
}

pub struct OrangeHudApp {
    engine: Arc<TelemetryEngine>,
    current_mode: HudViewMode,
    is_always_on_top: bool,
    #[allow(dead_code)]
    pub click_through: bool,
    #[allow(dead_code)]
    pub config: AppConfig,
}

impl OrangeHudApp {
    pub fn new(engine: Arc<TelemetryEngine>, config: AppConfig) -> Self {
        Self::with_mode(engine, config, HudViewMode::Compact)
    }

    pub fn with_mode(
        engine: Arc<TelemetryEngine>,
        config: AppConfig,
        initial_mode: HudViewMode,
    ) -> Self {
        Self {
            engine,
            current_mode: initial_mode,
            is_always_on_top: config.always_on_top,
            click_through: config.click_through,
            config,
        }
    }

    pub fn set_mode(&mut self, mode: HudViewMode, ctx: &egui::Context) {
        self.current_mode = mode;
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(mode.window_size()));
    }
}

impl eframe::App for OrangeHudApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // Essencial para janela frameless transparente sem cantos pretos/cinzas
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Redesenho reativo de baixa frequência (2Hz) para consumo residual de GPU (< 0.05%)
        ctx.request_repaint_after(Duration::from_millis(500));

        let snapshot = self.engine.snapshot().read().clone();

        // Janela frameless translúcida com cantos arredondados e borda sutil Orange
        let frame = Frame::none()
            .fill(Theme::BG_WINDOW)
            .rounding(Rounding::same(12.0))
            .stroke(Theme::BORDER_STROKE)
            .inner_margin(Margin::same(12.0));

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            self.render_header(ui, ctx, &snapshot);

            ui.add_space(10.0);

            // Alertas ativos destacados caso existam
            if !snapshot.alerts.is_empty() {
                self.render_alert_banner(ui, &snapshot.alerts);
                ui.add_space(8.0);
            }

            match self.current_mode {
                HudViewMode::Compact => self.render_compact_view(ui, &snapshot),
                HudViewMode::Expanded => self.render_expanded_view(ui, &snapshot),
                HudViewMode::Diagnostic => self.render_diagnostic_view(ui, &snapshot),
            }
        });
    }
}

impl OrangeHudApp {
    fn render_header(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, snap: &SystemSnapshot) {
        // Zona de arraste registrada ANTES dos botões: widgets posteriores têm prioridade
        // de clique, então os botões continuam clicáveis por cima dela.
        let header_rect = egui::Rect::from_min_size(
            ui.cursor().min,
            Vec2::new(ui.available_width(), 20.0),
        );
        let drag_area = ui.interact(
            header_rect,
            ui.id().with("header_drag_zone"),
            egui::Sense::click_and_drag(),
        );
        if drag_area.drag_started_by(egui::PointerButton::Primary) {
            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }

        ui.horizontal(|ui| {
            // Ícone laranja + título
            {
                {
                    let (dot_rect, _) =
                        ui.allocate_exact_size(Vec2::new(14.0, 16.0), egui::Sense::hover());
                    // Anel externo sutil + ponto central laranja
                    ui.painter().circle_filled(
                        dot_rect.center(),
                        6.0,
                        Color32::from_rgba_premultiplied(90, 44, 8, 120),
                    );
                    ui.painter().circle_filled(
                        dot_rect.center(),
                        3.6,
                        Theme::ORANGE_ACCENT,
                    );

                    ui.label(
                        egui::RichText::new("ORANGE HUD")
                            .size(11.5)
                            .color(Theme::TEXT_PRIMARY)
                            .strong(),
                    );
                }
            }

            // Badge de Saúde
            let (h_text, h_col) = match snap.health.score {
                90..=100 => ("OK", Theme::STATUS_EXCELLENT),
                75..=89 => ("BOM", Theme::STATUS_NORMAL),
                50..=74 => ("ALERTA", Theme::STATUS_WARNING),
                _ => ("CRÍTICO", Theme::STATUS_CRITICAL),
            };
            badge(ui, h_text, h_col, Color32::WHITE);

            // Botões de modo à direita
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 4.0;

                // Fechar
                if header_pill_button(ui, "×", false, true)
                    .on_hover_text("Fechar Orange HUD")
                    .clicked()
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }

                // Alternar Pin (Always on top)
                if header_pill_button(ui, "PIN", self.is_always_on_top, false)
                    .on_hover_text(if self.is_always_on_top {
                        "Fixado no topo (clique para soltar)"
                    } else {
                        "Fixar janela no topo"
                    })
                    .clicked()
                {
                    self.is_always_on_top = !self.is_always_on_top;
                    let level = if self.is_always_on_top {
                        egui::viewport::WindowLevel::AlwaysOnTop
                    } else {
                        egui::viewport::WindowLevel::Normal
                    };
                    ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
                }

                // Modo Diagnóstico
                if header_pill_button(
                    ui,
                    "DIAG",
                    self.current_mode == HudViewMode::Diagnostic,
                    false,
                )
                .on_hover_text("Diagnóstico Técnico")
                .clicked()
                {
                    self.set_mode(HudViewMode::Diagnostic, ctx);
                }

                // Modo Expandido
                if header_pill_button(
                    ui,
                    "EXP",
                    self.current_mode == HudViewMode::Expanded,
                    false,
                )
                .on_hover_text("Visão Expandida com Gráficos")
                .clicked()
                {
                    self.set_mode(HudViewMode::Expanded, ctx);
                }

                // Modo Compacto
                if header_pill_button(
                    ui,
                    "MIN",
                    self.current_mode == HudViewMode::Compact,
                    false,
                )
                .on_hover_text("Visão Compacta")
                .clicked()
                {
                    self.set_mode(HudViewMode::Compact, ctx);
                }
            });
        });
    }

    fn render_alert_banner(&self, ui: &mut egui::Ui, alerts: &[ActiveAlert]) {
        section_card(ui, |ui| {
            for alert in alerts {
                let bg_col = match alert.severity {
                    AlertSeverity::Critical => Theme::STATUS_CRITICAL,
                    AlertSeverity::Warning => Theme::STATUS_WARNING,
                    AlertSeverity::Info => Theme::STATUS_NORMAL,
                };

                ui.horizontal(|ui| {
                    badge(ui, "ALERTA", bg_col, Color32::WHITE);
                    ui.label(
                        egui::RichText::new(&alert.message)
                            .size(10.5)
                            .color(Theme::TEXT_PRIMARY),
                    );
                });
            }
        });
    }

    fn render_metric_grid(&self, ui: &mut egui::Ui, snap: &SystemSnapshot) {
        let gap = 8.0;
        let card_width = ((ui.available_width() - gap) * 0.5).max(120.0);

        // Linha 1: CPU + RAM
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;

            let cpu_sub = snap
                .cpu
                .package_temperature_c
                .map(|t| format!("{:.0}°C", t))
                .unwrap_or_else(|| format!("{} MHz", snap.cpu.current_frequency_mhz));
            metric_tile(
                ui,
                "CPU",
                &format!("{:.1}%", snap.cpu.total_usage_percent),
                &cpu_sub,
                snap.cpu.total_usage_percent,
                Theme::usage_color(snap.cpu.total_usage_percent),
                card_width,
            );

            let ram_used_gb = snap.memory.used_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
            let ram_total_gb = snap.memory.total_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
            metric_tile(
                ui,
                "MEMÓRIA",
                &format!("{:.1}%", snap.memory.usage_percent),
                &format!("{:.1}/{:.0} GB", ram_used_gb, ram_total_gb),
                snap.memory.usage_percent,
                Theme::usage_color(snap.memory.usage_percent),
                card_width,
            );
        });

        ui.add_space(gap);

        // Linha 2: GPU + Energia
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;

            if let Some(gpu) = snap.gpus.first() {
                let gpu_temp_txt = gpu
                    .temperature_core_c
                    .map(|t| format!("{:.0}°C", t))
                    .unwrap_or_else(|| {
                        if gpu.vram_total_bytes > 0 {
                            format!(
                                "{}/{} MB",
                                gpu.vram_used_bytes / 1_048_576,
                                gpu.vram_total_bytes / 1_048_576
                            )
                        } else {
                            "Ativa".to_string()
                        }
                    });
                metric_tile(
                    ui,
                    "GPU",
                    &format!("{:.1}%", gpu.usage_percent),
                    &gpu_temp_txt,
                    gpu.usage_percent,
                    Theme::usage_color(gpu.usage_percent),
                    card_width,
                );
            } else {
                metric_tile(ui, "GPU", "N/A", "--", 0.0, Theme::TEXT_MUTED, card_width);
            }

            let watts_txt = format!("{:.0} W", snap.energy.system_estimated_watts);
            let cost_txt = format!("R$ {:.2}", snap.energy.cumulative_cost_brl);
            let watts_percent = (snap.energy.system_estimated_watts / 250.0) * 100.0;
            metric_tile(
                ui,
                "POTÊNCIA",
                &watts_txt,
                &cost_txt,
                watts_percent,
                Theme::ORANGE_ACCENT,
                card_width,
            );
        });
    }

    fn render_compact_view(&self, ui: &mut egui::Ui, snap: &SystemSnapshot) {
        self.render_metric_grid(ui, snap);

        ui.add_space(8.0);

        // Rodapé discreto com rede e saúde do sistema
        let rx_mb = snap.network.download_bytes_sec as f32 / (1024.0 * 1024.0);
        let tx_mb = snap.network.upload_bytes_sec as f32 / (1024.0 * 1024.0);
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("REDE  ↓ {:.2}  ↑ {:.2} MB/s", rx_mb, tx_mb))
                    .size(9.5)
                    .color(Theme::TEXT_MUTED),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!("Saúde {}/100", snap.health.score))
                        .size(9.5)
                        .color(Theme::TEXT_SECONDARY),
                );
            });
        });
    }

    fn render_expanded_view(&self, ui: &mut egui::Ui, snap: &SystemSnapshot) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.render_metric_grid(ui, snap);

            ui.add_space(8.0);

            // Gráficos de Histórico em Tempo Real
            section_card(ui, |ui| {
                ui.label(
                    egui::RichText::new("HISTÓRICO RECENTE (60s)")
                        .size(10.0)
                        .color(Theme::ORANGE_ACCENT)
                        .strong(),
                );
                ui.add_space(6.0);

                let cpu_history = self.engine.cpu_history().read().to_chronological_vec();
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Carga da CPU").size(10.0).color(Theme::TEXT_SECONDARY));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{:.1}%", snap.cpu.total_usage_percent))
                                .size(10.0)
                                .color(Theme::ORANGE_ACCENT)
                                .strong(),
                        );
                    });
                });
                sparkline(ui, &cpu_history, 100.0, 38.0, Theme::ORANGE_ACCENT);

                ui.add_space(6.0);

                let ram_history = self.engine.ram_history().read().to_chronological_vec();
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Uso de RAM").size(10.0).color(Theme::TEXT_SECONDARY));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{:.1}%", snap.memory.usage_percent))
                                .size(10.0)
                                .color(Theme::STATUS_NORMAL)
                                .strong(),
                        );
                    });
                });
                sparkline(ui, &ram_history, 100.0, 38.0, Theme::STATUS_NORMAL);
            });

            ui.add_space(8.0);

            // Subsistema de Armazenamento e Rede
            section_card(ui, |ui| {
                ui.label(
                    egui::RichText::new("SUBSISTEMAS & REDE")
                        .size(10.0)
                        .color(Theme::ORANGE_ACCENT)
                        .strong(),
                );
                ui.add_space(4.0);

                let rx_mb = snap.network.download_bytes_sec as f32 / (1024.0 * 1024.0);
                let tx_mb = snap.network.upload_bytes_sec as f32 / (1024.0 * 1024.0);
                stat_row(
                    ui,
                    "Rede (Down / Up)",
                    &format!("{:.2} MB/s ↓ / {:.2} MB/s ↑", rx_mb, tx_mb),
                );
                stat_row(ui, "Interface Ativa", &snap.network.active_interface);
                stat_row(ui, "IP Local", &snap.network.local_ip);

                if let Some(disk) = snap.disks.first() {
                    let used_gb = disk.used_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
                    let total_gb = disk.total_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
                    stat_row(
                        ui,
                        &format!("Disco ({})", disk.mount_point),
                        &format!("{:.0}/{:.0} GB ({:.0}%)", used_gb, total_gb, disk.usage_percent),
                    );
                }
            });

            ui.add_space(8.0);

            // Top Processos
            section_card(ui, |ui| {
                ui.label(
                    egui::RichText::new("TOP PROCESSOS (CPU / RAM)")
                        .size(10.0)
                        .color(Theme::ORANGE_ACCENT)
                        .strong(),
                );
                ui.add_space(4.0);

                for proc in snap.top_processes.iter().take(4) {
                    let mem_mb = proc.memory_bytes as f32 / (1024.0 * 1024.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(&proc.name)
                                .size(10.5)
                                .color(Theme::TEXT_PRIMARY),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(format!("{:.1}% | {:.0} MB", proc.cpu_percent, mem_mb))
                                    .size(10.0)
                                    .color(Theme::TEXT_MUTED),
                            );
                        });
                    });
                }
            });
        });
    }

    fn render_diagnostic_view(&self, ui: &mut egui::Ui, snap: &SystemSnapshot) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // 1. Integridade de Hardware
            section_card(ui, |ui| {
                ui.label(
                    egui::RichText::new("DIAGNÓSTICO TÉCNICO & INTEGRIDADE")
                        .size(10.5)
                        .color(Theme::ORANGE_ACCENT)
                        .strong(),
                );
                ui.add_space(5.0);

                stat_row(
                    ui,
                    "Índice de Saúde Global",
                    &format!("{}/100 ({})", snap.health.score, snap.health.category.label()),
                );
                stat_row(
                    ui,
                    "Erros de Hardware (WHEA/MCE)",
                    &format!("{}", snap.health.whea_errors_detected),
                );
                stat_row(
                    ui,
                    "Thermal Throttling CPU",
                    if snap.cpu.thermal_throttled {
                        "ATIVO (PERIGO)"
                    } else {
                        "Inativo (Nominal)"
                    },
                );
                stat_row(
                    ui,
                    "Power Throttling CPU",
                    if snap.cpu.power_throttled {
                        "ATIVO"
                    } else {
                        "Inativo"
                    },
                );
            });

            ui.add_space(8.0);

            // 2. Detalhes de Processador e Arquitetura
            section_card(ui, |ui| {
                ui.label(
                    egui::RichText::new("PROCESSADOR")
                        .size(10.0)
                        .color(Theme::ORANGE_ACCENT)
                        .strong(),
                );
                ui.add_space(4.0);
                stat_row(ui, "Modelo", &snap.cpu.brand);
                stat_row(
                    ui,
                    "Núcleos Físicos / Threads",
                    &format!("{} / {}", snap.cpu.physical_cores, snap.cpu.logical_threads),
                );
                stat_row(ui, "Clock Atual", &format!("{} MHz", snap.cpu.current_frequency_mhz));
                stat_row(
                    ui,
                    "Load Average (1, 5, 15m)",
                    &format!(
                        "{:.2}, {:.2}, {:.2}",
                        snap.cpu.load_average.0, snap.cpu.load_average.1, snap.cpu.load_average.2
                    ),
                );
            });

            ui.add_space(8.0);

            // 3. Eficiência Energética e Bateria
            section_card(ui, |ui| {
                ui.label(
                    egui::RichText::new("ENERGIA & CUSTO")
                        .size(10.0)
                        .color(Theme::ORANGE_ACCENT)
                        .strong(),
                );
                ui.add_space(4.0);
                stat_row(
                    ui,
                    "Consumo Instantâneo",
                    &format!("{:.1} Watts", snap.energy.system_estimated_watts),
                );
                stat_row(
                    ui,
                    "Origem da Leitura",
                    if snap.energy.is_measured {
                        "Sensor Físico (Medido)"
                    } else {
                        "Modelo Matemático Estimado"
                    },
                );
                stat_row(
                    ui,
                    "Energia Acumulada",
                    &format!(
                        "{:.3} kWh ({:.1} Wh)",
                        snap.energy.cumulative_watt_hours / 1000.0,
                        snap.energy.cumulative_watt_hours
                    ),
                );
                stat_row(
                    ui,
                    "Custo Financeiro Estimado",
                    &format!("R$ {:.3}", snap.energy.cumulative_cost_brl),
                );

                if let Some(bat_pct) = snap.energy.battery_percent {
                    stat_row(
                        ui,
                        "Bateria",
                        &format!(
                            "{:.0}% ({})",
                            bat_pct,
                            if snap.energy.on_battery {
                                "Descarregando"
                            } else {
                                "Conectado à Tomada"
                            }
                        ),
                    );
                }
            });

            ui.add_space(8.0);

            // 4. Fatores Avaliados pelo Motor
            section_card(ui, |ui| {
                ui.label(
                    egui::RichText::new("FATORES DE INTEGRIDADE")
                        .size(10.0)
                        .color(Theme::ORANGE_ACCENT)
                        .strong(),
                );
                ui.add_space(4.0);
                for factor in &snap.health.factors {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(egui::RichText::new("•").size(10.0).color(Theme::ORANGE_ACCENT));
                        ui.label(egui::RichText::new(factor).size(10.0).color(Theme::TEXT_PRIMARY));
                    });
                }
            });
        });
    }
}
