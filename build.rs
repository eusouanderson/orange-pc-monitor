fn main() {
    // Apenas compila recursos (.rc / .ico) quando o alvo é Windows
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("FileDescription", "Orange PC Monitor");
        res.set("ProductName", "Orange PC Monitor");
        res.set("OriginalFilename", "orange-pc-monitor.exe");
        res.set("LegalCopyright", "Copyright (C) Orange Framework");

        if let Err(e) = res.compile() {
            eprintln!("cargo:warning=Aviso: Não foi possível compilar o recurso .rc (normal em cross-compile Linux): {}", e);
        }
    }
}
