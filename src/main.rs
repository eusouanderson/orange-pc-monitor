#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
//! Ponto de entrada do Orange PC Monitor (CLI / HUD Desktop / Benchmark).

use orange_pc_monitor::config::AppConfig;
use orange_pc_monitor::core::TelemetryEngine;
use orange_pc_monitor::ui::{HudViewMode, OrangeHudApp, Theme};
use std::env;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[cfg(target_os = "windows")]
fn attach_parent_console_if_needed() {
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::Console::{
        AttachConsole, GetStdHandle, ATTACH_PARENT_PROCESS, STD_OUTPUT_HANDLE,
    };

    unsafe {
        // Se stdout já estiver redirecionado (ex: pipes em testes/scripts), preserva o handle.
        let stdout = GetStdHandle(STD_OUTPUT_HANDLE).unwrap_or(HANDLE::default());
        if stdout.is_invalid() || stdout.0.is_null() {
            let _ = AttachConsole(ATTACH_PARENT_PROCESS);
        }
    }
}

fn print_banner() {
    println!("\x1b[1;38;5;208m");
    println!("   🍊 ORANGE PC MONITOR — ULTRA LOW RESOURCE SYSTEM HUD");
    println!("   Status: Ativo | Alvo de Consumo: < 30 MB RAM | < 0.2% CPU");
    println!("\x1b[0m");
}

fn print_snapshot_cli(engine: &TelemetryEngine, mode: &str) {
    let snap = engine.snapshot().read().clone();

    // Limpa terminal para exibição de HUD em tempo real
    print!("\x1b[2J\x1b[H");

    println!("=================================================================");
    println!(" 🍊 ORANGE PC MONITOR — HUD NATIVO");
    println!("=================================================================");

    let cpu = &snap.cpu;
    let mem = &snap.memory;
    let energy = &snap.energy;
    let net = &snap.network;
    let health = &snap.health;

    let total_gb = mem.total_bytes as f64 / 1_073_741_824.0;
    let used_gb = mem.used_bytes as f64 / 1_073_741_824.0;

    println!(
        " ⚡ CPU:   {:>5.1}% | {:>4} MHz | Temp: {:>5} | {}",
        cpu.total_usage_percent,
        cpu.current_frequency_mhz,
        cpu.package_temperature_c
            .map(|t| format!("{:.1}°C", t))
            .unwrap_or_else(|| "N/D".to_string()),
        cpu.brand
    );

    println!(
        " 🧠 RAM:   {:>5.1}% | {:.2} / {:.2} GB | Swap: {:.1}%",
        mem.usage_percent, used_gb, total_gb, mem.swap_percent
    );

    for (i, gpu) in snap.gpus.iter().enumerate() {
        let vram_used_mb = gpu.vram_used_bytes / 1_048_576;
        let vram_total_mb = gpu.vram_total_bytes / 1_048_576;
        println!(
            " 🎮 GPU{}:  {:>5.1}% | Temp: {:>5} | VRAM: {}/{} MB | {}",
            i,
            gpu.usage_percent,
            gpu.temperature_core_c
                .map(|t| format!("{:.1}°C", t))
                .unwrap_or_else(|| "N/D".to_string()),
            vram_used_mb,
            vram_total_mb,
            gpu.name
        );
    }

    let rx_mb = net.download_bytes_sec as f64 / 1_048_576.0;
    let tx_mb = net.upload_bytes_sec as f64 / 1_048_576.0;
    println!(
        " 🌐 REDE:  ↓ {:.2} MB/s | ↑ {:.2} MB/s | IP: {} ({})",
        rx_mb, tx_mb, net.local_ip, net.active_interface
    );

    println!(
        " 🔋 ENERGIA: {:.1} W (Est.) | Acumulado: {:.3} Wh | Custo: R$ {:.4}",
        energy.system_estimated_watts,
        energy.cumulative_watt_hours,
        energy.cumulative_cost_brl
    );

    println!(
        " 🩺 SAÚDE:  {} (Score: {}/100 | WHEA: {})",
        health.category.label(),
        health.score,
        health.whea_errors_detected
    );

    if !snap.alerts.is_empty() {
        println!("\n🚨 ALERTAS ATIVOS:");
        for alert in &snap.alerts {
            println!(" • [{:?}] {}: {}", alert.severity, alert.title, alert.message);
        }
    }

    if mode == "expanded" || mode == "diag" {
        println!("\n------------------ DISCOS & ARMAZENAMENTO -----------------------");
        for d in &snap.disks {
            let total_gb = d.total_bytes as f64 / 1_073_741_824.0;
            let used_gb = d.used_bytes as f64 / 1_073_741_824.0;
            let disk_type = if d.is_nvme {
                "NVMe"
            } else if d.is_ssd {
                "SSD"
            } else {
                "HDD"
            };
            println!(
                " • [{}] {} ({:.1} GB / {:.1} GB - {:.1}%)",
                disk_type, d.mount_point, used_gb, total_gb, d.usage_percent
            );
        }

        println!("\n------------------ TOP CONSUMO DE PROCESSOS ---------------------");
        for (idx, p) in snap.top_processes.iter().take(5).enumerate() {
            let mem_mb = p.memory_bytes / 1_048_576;
            println!(
                " {}. [PID {:>5}] {:<22} | CPU: {:>5.1}% | RAM: {:>4} MB",
                idx + 1,
                p.pid,
                p.name,
                p.cpu_percent,
                mem_mb
            );
        }

        println!("\n------------------ FATORES DE DIAGNÓSTICO -----------------------");
        for factor in &health.factors {
            println!(" • {}", factor);
        }
    }

    println!("=================================================================");
    println!(" [Pressione Ctrl+C para encerrar o monitor nativo]");
}

fn run_benchmark(duration_secs: u64) {
    println!("=================================================================");
    println!(" 🚀 INICIANDO BENCHMARK DE CONSUMO EXTREMO & LATÊNCIA");
    println!(" Duração da Amostragem: {} segundos", duration_secs);
    println!("=================================================================");

    let config = AppConfig::default();
    let mut engine = TelemetryEngine::new(config);
    engine.start();

    let start = Instant::now();
    let pid = sysinfo::Pid::from_u32(std::process::id());
    let mut sys = sysinfo::System::new();

    let mut rss_samples = Vec::new();
    let mut cpu_samples = Vec::new();

    while start.elapsed().as_secs() < duration_secs {
        thread::sleep(Duration::from_millis(500));
        sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);

        if let Some(proc) = sys.process(pid) {
            let rss_mb = proc.memory() as f64 / 1_048_576.0;
            let cpu_pct = proc.cpu_usage();
            rss_samples.push(rss_mb);
            cpu_samples.push(cpu_pct);
        }
    }

    engine.stop();

    let avg_rss = if !rss_samples.is_empty() {
        rss_samples.iter().sum::<f64>() / rss_samples.len() as f64
    } else {
        0.0
    };
    let max_rss = rss_samples.iter().cloned().fold(0.0_f64, f64::max);

    let avg_cpu = if !cpu_samples.is_empty() {
        cpu_samples.iter().sum::<f32>() / cpu_samples.len() as f32
    } else {
        0.0
    };

    println!("\n================== RELATÓRIO DE BENCHMARK =======================");
    println!(" • RAM VmRSS Média:      {:.2} MB (Alvo: < 30 MB)", avg_rss);
    println!(" • RAM VmRSS Pico:       {:.2} MB", max_rss);
    println!(" • CPU Uso Médio:        {:.3}% (Alvo: < 0.20%)", avg_cpu);
    println!("-----------------------------------------------------------------");

    let ram_passed = max_rss < 30.0;
    let cpu_passed = (avg_cpu as f64) < 0.50; // Margem segura

    if ram_passed {
        println!(" ✅ [PASS] Consumo de memória RAM cumpriu a meta rigorosa!");
    } else {
        println!(" ❌ [FAIL] Consumo de memória RAM excedeu 30 MB!");
    }

    if cpu_passed {
        println!(" ✅ [PASS] Consumo de CPU cumpriu a meta de ultrabaixo overhead!");
    } else {
        println!(" ❌ [FAIL] Consumo de CPU excedeu 0.50%!");
    }
    println!("=================================================================\n");
}

fn run_gui(engine: Arc<TelemetryEngine>, config: AppConfig, initial_mode: HudViewMode) {
    let initial_size = initial_mode.window_size();
    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_title("Orange PC Monitor")
        .with_inner_size([initial_size.x, initial_size.y])
        .with_min_inner_size([320.0, 210.0])
        .with_decorations(false)
        .with_transparent(true)
        .with_resizable(true);

    if config.always_on_top {
        viewport = viewport.with_always_on_top();
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    let app = OrangeHudApp::with_mode(engine, config, initial_mode);
    if let Err(err) = eframe::run_native(
        "Orange PC Monitor",
        native_options,
        Box::new(|cc| {
            Theme::apply(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    ) {
        eprintln!("Erro ao iniciar interface gráfica do Orange PC Monitor: {err}");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let wants_version = args.iter().any(|a| a == "--version" || a == "-v");
    let wants_benchmark = args.iter().any(|a| a == "--benchmark");
    let wants_json = args.iter().any(|a| a == "--json");
    let wants_once = args.iter().any(|a| a == "--once");
    let wants_cli = args.iter().any(|a| a == "--cli");
    let wants_gui = args.iter().any(|a| a == "--gui");

    let is_cli_invocation =
        wants_version || wants_benchmark || wants_json || wants_once || wants_cli;

    #[cfg(target_os = "windows")]
    if is_cli_invocation {
        attach_parent_console_if_needed();
    }

    if wants_version {
        println!("orange-pc-monitor 0.1.0 (Orange Framework Native Core)");
        return;
    }

    if wants_benchmark {
        let duration = args
            .iter()
            .position(|a| a == "--benchmark")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(4);
        run_benchmark(duration);
        return;
    }

    let config = AppConfig::load_or_default();
    let mut engine = TelemetryEngine::new(config.clone());
    engine.start();

    let (mode, initial_hud_mode) = if args.iter().any(|a| a == "--diag") {
        ("diag", HudViewMode::Diagnostic)
    } else if args.iter().any(|a| a == "--expanded") {
        ("expanded", HudViewMode::Expanded)
    } else {
        ("compact", HudViewMode::Compact)
    };

    // Em Windows e macOS o ambiente gráfico é nativo; em Linux verifica DISPLAY / WAYLAND_DISPLAY
    let has_display = cfg!(any(target_os = "windows", target_os = "macos"))
        || env::var("DISPLAY").is_ok()
        || env::var("WAYLAND_DISPLAY").is_ok();

    if wants_gui || (has_display && !is_cli_invocation) {
        let arc_engine = Arc::new(engine);
        run_gui(arc_engine, config, initial_hud_mode);
        return;
    }

    print_banner();

    // Aguarda primeiro ciclo de amostragem para saída CLI/JSON consistente
    thread::sleep(Duration::from_millis(600));

    if wants_json {
        let snap = engine.snapshot().read().clone();
        println!(
            "{}",
            serde_json::to_string_pretty(&snap).unwrap_or_default()
        );
        return;
    }

    if wants_once {
        print_snapshot_cli(&engine, mode);
        return;
    }

    // Modo CLI padrão (loop interativo)
    loop {
        print_snapshot_cli(&engine, mode);
        thread::sleep(Duration::from_millis(1000));
    }
}
