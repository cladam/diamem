use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_title("diamem — Live Diagram Editor"),
        ..Default::default()
    };

    eframe::run_native(
        "diamem",
        options,
        Box::new(|cc| Ok(Box::new(diamem::DiamemApp::new(cc)))),
    )
}
