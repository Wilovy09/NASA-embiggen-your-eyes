use colorgrad::Gradient;
use eframe::egui;
use egui::{Color32, ColorImage, Pos2, Rect, Stroke, TextureHandle, Ui, Vec2};
use fitsio::FitsFile;
use serde::{Deserialize, Serialize};

// Estructura principal de la aplicación DS9-like
struct DS9App {
    // Datos de imagen FITS
    image_data: Vec<f32>,
    img_width: usize,
    img_height: usize,
    min_val: f32,
    max_val: f32,

    // Control de visualización
    zoom: f32,
    pan_x: f32,
    pan_y: f32,
    rotation: f32,
    flip_x: bool,
    flip_y: bool,

    // Procesamiento de imagen
    color_map: ColorMap,
    scaling: ScalingMode,
    contrast: f32,
    brightness: f32,
    gamma: f32,
    invert: bool,

    // Textura para mostrar la imagen
    texture: Option<TextureHandle>,

    // Optimización de rendimiento
    texture_needs_update: bool,
    last_color_map: ColorMap,
    last_scaling: ScalingMode,
    last_contrast: f32,
    last_brightness: f32,
    last_gamma: f32,
    last_invert: bool,

    // Downsampling para mejor rendimiento
    display_width: usize,
    display_height: usize,
    downsample_factor: usize,

    // Estado de la interfaz
    show_info_panel: bool,
    show_histogram: bool,
    show_crosshair: bool,
    show_regions: bool,
    show_wcs: bool,

    // Regiones de interés
    regions: Vec<Region>,
    current_region_type: RegionType,

    // Cursor y coordenadas
    cursor_pos: Option<Pos2>,
    cursor_value: f32,
    cursor_coords: (f32, f32),

    // Histograma
    histogram: Vec<u32>,
    histogram_bins: usize,

    // Archivo actual
    current_file: String,

    // Control de ventanas
    window_size: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
enum ColorMap {
    Grayscale,
    Heat,
    Cool,
    Rainbow,
    Viridis,
    Plasma,
    DS9A,
    DS9B,
    DS9BB,
    DS9He,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
enum ScalingMode {
    Linear,
    Log,
    Sqrt,
    Squared,
    ArcSinh,
    HistEq,
    ZScale,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum RegionType {
    Circle,
    Rectangle,
    Point,
    Line,
    Polygon,
}

#[derive(Debug, Clone, PartialEq)]
struct Region {
    region_type: RegionType,
    center: Pos2,
    size: Vec2,
    points: Vec<Pos2>,
    color: Color32,
    label: String,
}

impl ColorMap {
    fn all() -> Vec<ColorMap> {
        vec![
            ColorMap::Grayscale,
            ColorMap::Heat,
            ColorMap::Cool,
            ColorMap::Rainbow,
            ColorMap::Viridis,
            ColorMap::Plasma,
            ColorMap::DS9A,
            ColorMap::DS9B,
            ColorMap::DS9BB,
            ColorMap::DS9He,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            ColorMap::Grayscale => "Grayscale",
            ColorMap::Heat => "Heat",
            ColorMap::Cool => "Cool",
            ColorMap::Rainbow => "Rainbow",
            ColorMap::Viridis => "Viridis",
            ColorMap::Plasma => "Plasma",
            ColorMap::DS9A => "DS9 A",
            ColorMap::DS9B => "DS9 B",
            ColorMap::DS9BB => "DS9 BB",
            ColorMap::DS9He => "DS9 He",
        }
    }

    fn get_gradient(&self) -> Box<dyn Gradient> {
        match self {
            ColorMap::Grayscale => Box::new(colorgrad::preset::greys()),
            ColorMap::Heat => Box::new(colorgrad::preset::turbo()),
            ColorMap::Cool => Box::new(colorgrad::preset::cool()),
            ColorMap::Rainbow => Box::new(colorgrad::preset::rainbow()),
            ColorMap::Viridis => Box::new(colorgrad::preset::viridis()),
            ColorMap::Plasma => Box::new(colorgrad::preset::plasma()),
            ColorMap::DS9A => Box::new(colorgrad::preset::inferno()),
            ColorMap::DS9B => Box::new(colorgrad::preset::magma()),
            ColorMap::DS9BB => Box::new(colorgrad::preset::plasma()),
            ColorMap::DS9He => Box::new(colorgrad::preset::cividis()),
        }
    }
}

impl ScalingMode {
    fn all() -> Vec<ScalingMode> {
        vec![
            ScalingMode::Linear,
            ScalingMode::Log,
            ScalingMode::Sqrt,
            ScalingMode::Squared,
            ScalingMode::ArcSinh,
            ScalingMode::HistEq,
            ScalingMode::ZScale,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            ScalingMode::Linear => "Linear",
            ScalingMode::Log => "Log",
            ScalingMode::Sqrt => "Sqrt",
            ScalingMode::Squared => "Squared",
            ScalingMode::ArcSinh => "ArcSinh",
            ScalingMode::HistEq => "Hist Eq",
            ScalingMode::ZScale => "ZScale",
        }
    }

    fn apply(&self, value: f32, min_val: f32, max_val: f32) -> f32 {
        let normalized = ((value - min_val) / (max_val - min_val)).clamp(0.0, 1.0);

        match self {
            ScalingMode::Linear => normalized,
            ScalingMode::Log => {
                if normalized <= 0.0 {
                    0.0
                } else {
                    (1.0 + normalized * 999.0).ln() / 1000.0_f32.ln()
                }
            }
            ScalingMode::Sqrt => normalized.sqrt(),
            ScalingMode::Squared => normalized * normalized,
            ScalingMode::ArcSinh => {
                let scaled = normalized * 10.0;
                scaled.asinh() / 10.0_f32.asinh()
            }
            ScalingMode::HistEq => normalized.powf(0.5), // Simplified histogram equalization
            ScalingMode::ZScale => {
                // Simplified ZScale - in real implementation would use statistics
                let z1 = 0.1;
                let z2 = 0.9;
                ((normalized - z1) / (z2 - z1)).clamp(0.0, 1.0)
            }
        }
    }
}

impl Default for DS9App {
    fn default() -> Self {
        Self {
            image_data: Vec::new(),
            img_width: 0,
            img_height: 0,
            min_val: 0.0,
            max_val: 1.0,
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            color_map: ColorMap::Grayscale,
            scaling: ScalingMode::Linear,
            contrast: 1.0,
            brightness: 0.0,
            gamma: 1.0,
            invert: false,
            texture: None,
            texture_needs_update: true,
            last_color_map: ColorMap::Grayscale,
            last_scaling: ScalingMode::Linear,
            last_contrast: 1.0,
            last_brightness: 0.0,
            last_gamma: 1.0,
            last_invert: false,
            display_width: 0,
            display_height: 0,
            downsample_factor: 1,
            show_info_panel: true,
            show_histogram: false,
            show_crosshair: false,
            show_regions: true,
            show_wcs: false,
            regions: Vec::new(),
            current_region_type: RegionType::Circle,
            cursor_pos: None,
            cursor_value: 0.0,
            cursor_coords: (0.0, 0.0),
            histogram: Vec::new(),
            histogram_bins: 256,
            current_file: String::new(),
            window_size: Vec2::new(1200.0, 800.0),
        }
    }
}

impl DS9App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();

        // Intentar cargar una imagen FITS por defecto
        if let Ok(_) = app.load_fits_file("h_m51_b_s05_drz_sci.fits") {
            println!("Imagen FITS cargada exitosamente");
        }

        app
    }

    fn load_fits_file(&mut self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut fptr = FitsFile::open(filename)?;
        let hdu = fptr.primary_hdu()?;

        // Obtener dimensiones
        let (width, height) = match &hdu.info {
            fitsio::hdu::HduInfo::ImageInfo { shape, .. } => (shape[1], shape[0]),
            _ => return Err("No es una imagen FITS válida".into()),
        };

        // Cargar datos
        let data: Vec<f32> = hdu.read_image(&mut fptr)?;

        // Calcular estadísticas
        let mut min_val = f32::MAX;
        let mut max_val = f32::MIN;

        for &val in &data {
            if val.is_finite() {
                min_val = min_val.min(val);
                max_val = max_val.max(val);
            }
        }

        // Calcular factor de downsampling para imágenes grandes
        let max_display_size = 2048; // Máximo tamaño de display
        self.downsample_factor = if width > max_display_size || height > max_display_size {
            ((width.max(height) as f32 / max_display_size as f32).ceil() as usize).max(1)
        } else {
            1
        };

        // Asegurar que las dimensiones de display sean válidas
        self.display_width = (width + self.downsample_factor - 1) / self.downsample_factor; // División con redondeo hacia arriba
        self.display_height = (height + self.downsample_factor - 1) / self.downsample_factor;

        // Actualizar estado de la aplicación
        self.image_data = data;
        self.img_width = width;
        self.img_height = height;
        self.min_val = min_val;
        self.max_val = max_val;
        self.current_file = filename.to_string();
        self.texture_needs_update = true;

        // Calcular histograma (usando muestreo para mejor rendimiento)
        self.calculate_histogram();

        // Resetear vista
        self.zoom = 1.0;
        self.pan_x = 0.0;
        self.pan_y = 0.0;

        println!(
            "Imagen cargada: {}x{} (display: {}x{}), rango: {:.3} - {:.3}, downsample: {}x",
            width,
            height,
            self.display_width,
            self.display_height,
            min_val,
            max_val,
            self.downsample_factor
        );

        Ok(())
    }

    fn calculate_histogram(&mut self) {
        self.histogram = vec![0; self.histogram_bins];

        // Muestrear cada N píxeles para mejor rendimiento en imágenes grandes
        let sample_step = if self.image_data.len() > 1_000_000 {
            10
        } else {
            1
        };

        for (i, &val) in self.image_data.iter().enumerate() {
            if i % sample_step != 0 {
                continue;
            }

            if val.is_finite() && val >= self.min_val && val <= self.max_val {
                let normalized = (val - self.min_val) / (self.max_val - self.min_val);
                let bin = ((normalized * (self.histogram_bins as f32 - 1.0)) as usize)
                    .min(self.histogram_bins - 1);
                self.histogram[bin] += sample_step as u32; // Compensar el muestreo
            }
        }
    }

    fn needs_texture_update(&self) -> bool {
        self.texture_needs_update
            || self.color_map != self.last_color_map
            || self.scaling != self.last_scaling
            || (self.contrast - self.last_contrast).abs() > 0.01
            || (self.brightness - self.last_brightness).abs() > 0.01
            || (self.gamma - self.last_gamma).abs() > 0.01
            || self.invert != self.last_invert
    }

    fn create_texture(&mut self, ctx: &egui::Context) {
        if self.image_data.is_empty() || !self.needs_texture_update() {
            return;
        }

        let mut color_data = Vec::with_capacity(self.display_width * self.display_height * 4);
        let gradient = self.color_map.get_gradient();

        // Usar downsampling para mejor rendimiento
        let ds = self.downsample_factor;

        // Asegurar que el tamaño calculado sea correcto
        let expected_pixels = self.display_width * self.display_height;
        color_data.reserve_exact(expected_pixels * 4);

        for dy in 0..self.display_height {
            for dx in 0..self.display_width {
                let y = dy * ds;
                let x = dx * ds;

                if y >= self.img_height || x >= self.img_width {
                    // Pixel fuera de límites - usar negro
                    color_data.extend_from_slice(&[0, 0, 0, 255]);
                    continue;
                }

                let idx = y * self.img_width + x;
                if idx >= self.image_data.len() {
                    // Índice fuera de límites - usar negro
                    color_data.extend_from_slice(&[0, 0, 0, 255]);
                    continue;
                }

                let val = self.image_data[idx];
                let mut processed_val = if val.is_finite() {
                    self.scaling.apply(val, self.min_val, self.max_val)
                } else {
                    0.0
                };

                // Aplicar ajustes
                processed_val =
                    ((processed_val - 0.5) * self.contrast + 0.5 + self.brightness).clamp(0.0, 1.0);
                processed_val = processed_val.powf(self.gamma);

                if self.invert {
                    processed_val = 1.0 - processed_val;
                }

                let color = gradient.at(processed_val);
                color_data.push((color.r as f32 * 255.0) as u8);
                color_data.push((color.g as f32 * 255.0) as u8);
                color_data.push((color.b as f32 * 255.0) as u8);
                color_data.push(255);
            }
        }

        let color_image = ColorImage::from_rgba_unmultiplied(
            [self.display_width, self.display_height],
            &color_data,
        );

        self.texture = Some(ctx.load_texture("fits_image", color_image, Default::default()));

        // Actualizar cache
        self.texture_needs_update = false;
        self.last_color_map = self.color_map;
        self.last_scaling = self.scaling;
        self.last_contrast = self.contrast;
        self.last_brightness = self.brightness;
        self.last_gamma = self.gamma;
        self.last_invert = self.invert;
    }

    fn menu_bar(&mut self, ui: &mut Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open FITS...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("FITS files", &["fits", "fit", "fts"])
                        .pick_file()
                    {
                        if let Some(path_str) = path.to_str() {
                            let _ = self.load_fits_file(path_str);
                        }
                    }
                    ui.close_menu();
                }
                if ui.button("Save Image...").clicked() {
                    // TODO: Implementar guardado
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Exit").clicked() {
                    std::process::exit(0);
                }
            });

            ui.menu_button("View", |ui| {
                ui.checkbox(&mut self.show_info_panel, "Info Panel");
                ui.checkbox(&mut self.show_histogram, "Histogram");
                ui.checkbox(&mut self.show_crosshair, "Crosshair");
                ui.checkbox(&mut self.show_regions, "Regions");
                ui.checkbox(&mut self.show_wcs, "WCS Coordinates");
                ui.separator();
                if ui.button("Zoom to Fit").clicked() {
                    self.zoom = 1.0;
                    self.pan_x = 0.0;
                    self.pan_y = 0.0;
                    ui.close_menu();
                }
                if ui.button("Zoom 1:1").clicked() {
                    self.zoom = 1.0;
                    ui.close_menu();
                }
            });

            ui.menu_button("Scale", |ui| {
                for scale in ScalingMode::all() {
                    if ui
                        .selectable_label(self.scaling == scale, scale.name())
                        .clicked()
                    {
                        self.scaling = scale;
                        self.texture_needs_update = true;
                        ui.close_menu();
                    }
                }
            });

            ui.menu_button("Color", |ui| {
                for cmap in ColorMap::all() {
                    if ui
                        .selectable_label(self.color_map == cmap, cmap.name())
                        .clicked()
                    {
                        self.color_map = cmap;
                        self.texture_needs_update = true;
                        ui.close_menu();
                    }
                }
                ui.separator();
                if ui.checkbox(&mut self.invert, "Invert").changed() {
                    self.texture_needs_update = true;
                }
            });

            ui.menu_button("Region", |ui| {
                ui.selectable_value(&mut self.current_region_type, RegionType::Circle, "Circle");
                ui.selectable_value(
                    &mut self.current_region_type,
                    RegionType::Rectangle,
                    "Rectangle",
                );
                ui.selectable_value(&mut self.current_region_type, RegionType::Point, "Point");
                ui.selectable_value(&mut self.current_region_type, RegionType::Line, "Line");
                ui.separator();
                if ui.button("Clear All Regions").clicked() {
                    self.regions.clear();
                    ui.close_menu();
                }
            });
        });
    }

    fn side_panel(&mut self, ctx: &egui::Context) {
        if !self.show_info_panel {
            return;
        }

        egui::SidePanel::right("info_panel")
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("Image Info");

                ui.separator();

                // Información de archivo
                ui.label(format!("File: {}", self.current_file));
                ui.label(format!("Size: {}×{}", self.img_width, self.img_height));
                ui.label(format!("Range: {:.3} to {:.3}", self.min_val, self.max_val));

                ui.separator();

                // Controles de visualización
                ui.heading("Display");

                ui.horizontal(|ui| {
                    ui.label("Zoom:");
                    ui.add(
                        egui::DragValue::new(&mut self.zoom)
                            .range(0.1..=10.0)
                            .speed(0.1),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Pan X:");
                    ui.add(egui::DragValue::new(&mut self.pan_x).speed(1.0));
                });

                ui.horizontal(|ui| {
                    ui.label("Pan Y:");
                    ui.add(egui::DragValue::new(&mut self.pan_y).speed(1.0));
                });

                ui.horizontal(|ui| {
                    ui.label("Rotation:");
                    ui.add(
                        egui::DragValue::new(&mut self.rotation)
                            .range(0.0..=360.0)
                            .speed(1.0),
                    );
                });

                ui.checkbox(&mut self.flip_x, "Flip X");
                ui.checkbox(&mut self.flip_y, "Flip Y");

                ui.separator();

                // Controles de imagen
                ui.heading("Image Processing");

                ui.horizontal(|ui| {
                    ui.label("Brightness:");
                    let response =
                        ui.add(egui::Slider::new(&mut self.brightness, -0.5..=0.5).step_by(0.01));
                    if response.changed() {
                        self.texture_needs_update = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Contrast:");
                    let response =
                        ui.add(egui::Slider::new(&mut self.contrast, 0.1..=3.0).step_by(0.1));
                    if response.changed() {
                        self.texture_needs_update = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Gamma:");
                    let response =
                        ui.add(egui::Slider::new(&mut self.gamma, 0.1..=3.0).step_by(0.1));
                    if response.changed() {
                        self.texture_needs_update = true;
                    }
                });
                ui.separator();

                // Información del cursor
                ui.heading("Cursor Info");

                if let Some(pos) = self.cursor_pos {
                    ui.label(format!("Screen: ({:.0}, {:.0})", pos.x, pos.y));
                    ui.label(format!(
                        "Image: ({:.1}, {:.1})",
                        self.cursor_coords.0, self.cursor_coords.1
                    ));
                    ui.label(format!("Value: {:.3}", self.cursor_value));
                } else {
                    ui.label("No cursor position");
                }

                ui.separator();

                // Estadísticas de regiones
                if !self.regions.is_empty() {
                    ui.heading(format!("Regions ({})", self.regions.len()));

                    for (i, region) in self.regions.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}: {:?}", i + 1, region.region_type));
                            if ui.small_button("×").clicked() {
                                // TODO: Eliminar región
                            }
                        });
                    }
                }
            });
    }

    fn histogram_window(&mut self, ctx: &egui::Context) {
        if !self.show_histogram {
            return;
        }

        egui::Window::new("Histogram")
            .resizable(true)
            .default_size([400.0, 300.0])
            .show(ctx, |ui| {
                ui.label("Histogram (simplified view)");
                ui.separator();

                // Simple text-based histogram display
                let max_count = self.histogram.iter().max().unwrap_or(&1);
                for (i, &count) in self.histogram.iter().enumerate().take(32) {
                    let bar_length = (count as f32 / *max_count as f32 * 20.0) as usize;
                    let bar = "█".repeat(bar_length);
                    ui.horizontal(|ui| {
                        ui.label(format!("{:3}", i));
                        ui.label(bar);
                        ui.label(format!("{}", count));
                    });
                }
            });
    }

    fn image_viewer(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.image_data.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label("No FITS image loaded. Use File > Open FITS... to load an image.");
                });
                return;
            }

            // Crear textura si es necesario
            if self.texture.is_none() {
                self.create_texture(ctx);
            }

            let available_size = ui.available_size();
            let response = ui.allocate_response(available_size, egui::Sense::click_and_drag());

            if let Some(texture) = &self.texture {
                let image_size = Vec2::new(
                    self.img_width as f32 * self.zoom,
                    self.img_height as f32 * self.zoom,
                );

                let center = response.rect.center();
                let image_rect =
                    Rect::from_center_size(center + Vec2::new(self.pan_x, self.pan_y), image_size);

                // Dibujar imagen
                ui.painter().image(
                    texture.id(),
                    image_rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );

                // Manejar interacciones del mouse
                if response.hovered() {
                    if let Some(hover_pos) = response.hover_pos() {
                        self.cursor_pos = Some(hover_pos);

                        // Convertir coordenadas de pantalla a imagen
                        let rel_pos = hover_pos - image_rect.min;
                        let img_x = (rel_pos.x / self.zoom) as usize;
                        let img_y = (rel_pos.y / self.zoom) as usize;

                        if img_x < self.img_width && img_y < self.img_height {
                            let idx = img_y * self.img_width + img_x;
                            self.cursor_coords = (img_x as f32, img_y as f32);
                            self.cursor_value = self.image_data[idx];
                        }

                        // Dibujar crosshair
                        if self.show_crosshair {
                            let painter = ui.painter();
                            painter.line_segment(
                                [
                                    Pos2::new(hover_pos.x, response.rect.min.y),
                                    Pos2::new(hover_pos.x, response.rect.max.y),
                                ],
                                Stroke::new(1.0, Color32::RED),
                            );
                            painter.line_segment(
                                [
                                    Pos2::new(response.rect.min.x, hover_pos.y),
                                    Pos2::new(response.rect.max.x, hover_pos.y),
                                ],
                                Stroke::new(1.0, Color32::RED),
                            );
                        }
                    }
                }

                // Manejar zoom con rueda del mouse
                if response.hovered() {
                    let scroll = ui.input(|i| i.raw_scroll_delta.y);
                    if scroll != 0.0 {
                        let zoom_factor = 1.1;
                        if scroll > 0.0 {
                            self.zoom *= zoom_factor;
                        } else {
                            self.zoom /= zoom_factor;
                        }
                        self.zoom = self.zoom.clamp(0.1, 10.0);
                    }
                }

                // Manejar arrastre para pan
                if response.dragged() {
                    self.pan_x += response.drag_delta().x;
                    self.pan_y += response.drag_delta().y;
                }

                // Dibujar regiones
                if self.show_regions {
                    let painter = ui.painter();
                    for region in &self.regions {
                        match region.region_type {
                            RegionType::Circle => {
                                painter.circle_stroke(
                                    region.center,
                                    region.size.x,
                                    Stroke::new(2.0, region.color),
                                );
                            }
                            RegionType::Rectangle => {
                                painter.rect_stroke(
                                    Rect::from_center_size(region.center, region.size),
                                    0.0,
                                    Stroke::new(2.0, region.color),
                                );
                            }
                            RegionType::Point => {
                                painter.circle_filled(region.center, 3.0, region.color);
                            }
                            _ => {} // TODO: Implementar otros tipos
                        }
                    }
                }
            }

            // Manejar teclas
            ui.input(|i| {
                if i.key_pressed(egui::Key::Space) {
                    self.zoom = 1.0;
                    self.pan_x = 0.0;
                    self.pan_y = 0.0;
                }

                if i.key_pressed(egui::Key::R) {
                    self.rotation += 90.0;
                    if self.rotation >= 360.0 {
                        self.rotation = 0.0;
                    }
                }

                if i.key_pressed(egui::Key::X) {
                    self.flip_x = !self.flip_x;
                }

                if i.key_pressed(egui::Key::Y) {
                    self.flip_y = !self.flip_y;
                }

                if i.key_pressed(egui::Key::I) {
                    self.invert = !self.invert;
                }
            });
        });
    }
}

impl eframe::App for DS9App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Solo recrear textura si es necesario (gran optimización de rendimiento)
        self.create_texture(ctx);

        // Limitar framerate para mejor rendimiento
        ctx.request_repaint_after(std::time::Duration::from_millis(16)); // ~60 FPS

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            self.menu_bar(ui);
        });

        self.side_panel(ctx);
        if self.show_histogram {
            self.histogram_window(ctx);
        }
        self.image_viewer(ctx);
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("DS9-like FITS Viewer")
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "DS9-like FITS Viewer",
        options,
        Box::new(|cc| Ok(Box::new(DS9App::new(cc)))),
    )
}
