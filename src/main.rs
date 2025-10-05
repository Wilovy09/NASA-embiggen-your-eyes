use fitsio::FitsFile;
use minifb::{Key, Window, WindowOptions};

// Función simple para dibujar un pixel en el buffer
fn draw_pixel(buffer: &mut [u32], x: usize, y: usize, width: usize, color: u32) {
    if x < width && y * width + x < buffer.len() {
        buffer[y * width + x] = color;
    }
}

// Función para dibujar un rectángulo (para el fondo del texto)
fn draw_rect(buffer: &mut [u32], x: usize, y: usize, w: usize, h: usize, width: usize, color: u32) {
    for dy in 0..h {
        for dx in 0..w {
            draw_pixel(buffer, x + dx, y + dy, width, color);
        }
    }
}

// Función para dibujar texto simple (solo números básicos)
fn draw_char(buffer: &mut [u32], ch: char, x: usize, y: usize, width: usize, color: u32) {
    // Patrones simples de 5x7 píxeles para algunos caracteres
    let patterns: &[u8] = match ch {
        '0' => &[
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        '1' => &[
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => &[
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '3' => &[
            0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110,
        ],
        '4' => &[
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => &[
            0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110,
        ],
        '6' => &[
            0b01110, 0b10000, 0b11110, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        '7' => &[
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => &[
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => &[
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110,
        ],
        '.' => &[
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100,
        ],
        '-' => &[
            0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ],
        ':' => &[
            0b00000, 0b01100, 0b01100, 0b00000, 0b01100, 0b01100, 0b00000,
        ],
        ' ' => &[
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        'x' => &[
            0b00000, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b00000,
        ],
        '°' => &[
            0b01110, 0b10001, 0b10001, 0b01110, 0b00000, 0b00000, 0b00000,
        ],
        _ => &[
            0b11111, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11111,
        ], // Cuadrado para chars desconocidos
    };

    for (row, &pattern) in patterns.iter().enumerate() {
        for col in 0..5 {
            if (pattern >> (4 - col)) & 1 == 1 {
                draw_pixel(buffer, x + col, y + row, width, color);
            }
        }
    }
}

// Función para dibujar una cadena de texto
fn draw_text(buffer: &mut [u32], text: &str, x: usize, y: usize, width: usize, color: u32) {
    for (i, ch) in text.chars().enumerate() {
        draw_char(buffer, ch, x + i * 6, y, width, color);
    }
}

#[derive(Clone, Copy)]
enum BackgroundColor {
    Black,
    Gray,
    White,
}

#[derive(Clone, Copy)]
enum ProcessingMode {
    Linear,
    HistogramEqualized,
    PowerLaw,
    LogScale,
}

impl ProcessingMode {
    fn next(self) -> Self {
        match self {
            ProcessingMode::Linear => ProcessingMode::HistogramEqualized,
            ProcessingMode::HistogramEqualized => ProcessingMode::PowerLaw,
            ProcessingMode::PowerLaw => ProcessingMode::LogScale,
            ProcessingMode::LogScale => ProcessingMode::Linear,
        }
    }

    fn name(self) -> &'static str {
        match self {
            ProcessingMode::Linear => "Lineal",
            ProcessingMode::HistogramEqualized => "Ecualizado",
            ProcessingMode::PowerLaw => "Gamma",
            ProcessingMode::LogScale => "Logarítmico",
        }
    }
}

impl BackgroundColor {
    fn next(self) -> Self {
        match self {
            BackgroundColor::Black => BackgroundColor::Gray,
            BackgroundColor::Gray => BackgroundColor::White,
            BackgroundColor::White => BackgroundColor::Black,
        }
    }

    fn to_rgb(self) -> u32 {
        match self {
            BackgroundColor::Black => 0x000000,
            BackgroundColor::Gray => 0x808080,
            BackgroundColor::White => 0xFFFFFF,
        }
    }

    fn name(self) -> &'static str {
        match self {
            BackgroundColor::Black => "negro",
            BackgroundColor::Gray => "gris",
            BackgroundColor::White => "blanco",
        }
    }
}

// Obtiene las coordenadas rotadas
fn get_rotated_coords(
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    rotation: u8,
) -> (usize, usize) {
    match rotation {
        0 => (x, y),                          // Sin rotación
        1 => (height - 1 - y, x),             // 90° CW
        2 => (width - 1 - x, height - 1 - y), // 180°
        3 => (y, width - 1 - x),              // 270° CW (90° CCW)
        _ => (x, y),
    }
}

// Obtiene las dimensiones después de la rotación
fn get_rotated_dimensions(width: usize, height: usize, rotation: u8) -> (usize, usize) {
    match rotation {
        1 | 3 => (height, width), // 90° o 270°: intercambiar dimensiones
        _ => (width, height),     // 0° o 180°: mantener dimensiones
    }
}

// Función para ecualización de histograma simplificada
fn histogram_equalize(normalized: f32) -> f32 {
    // Aplica una función de ecualización que realza el contraste
    let enhanced = normalized.powf(0.5); // Raíz cuadrada para realzar detalles débiles
    enhanced.clamp(0.0, 1.0)
}

// Función para escala logarítmica
fn log_scale(normalized: f32) -> f32 {
    if normalized <= 0.0 {
        0.0
    } else {
        (1.0 + normalized * 999.0).ln() / 1000.0_f32.ln()
    }
}

// Convierte un valor flotante de FITS a u32 RGB con procesamiento avanzado
fn grayscale_to_rgb(
    val: f32,
    min_val: f32,
    max_val: f32,
    background_color: BackgroundColor,
    processing_mode: ProcessingMode,
    brightness: f32,
    contrast: f32,
    gamma: f32,
    inverted: bool,
) -> u32 {
    // Para datos astronómicos, usar un percentil en lugar del min/max absoluto
    // Esto ayuda con el rango dinámico extremo típico de FITS
    let effective_min = min_val;
    let effective_max = max_val * 0.1; // Usar solo el 10% superior del rango

    // Normalización con rango efectivo
    let mut normalized = if effective_max > effective_min {
        ((val - effective_min) / (effective_max - effective_min)).clamp(0.0, 1.0)
    } else {
        0.0
    };

    // Aplicar ajustes de brillo y contraste
    normalized = ((normalized - 0.5) * contrast + 0.5 + brightness).clamp(0.0, 1.0);

    // Aplicar procesamiento según el modo
    normalized = match processing_mode {
        ProcessingMode::Linear => normalized,
        ProcessingMode::HistogramEqualized => histogram_equalize(normalized),
        ProcessingMode::PowerLaw => normalized.powf(gamma),
        ProcessingMode::LogScale => log_scale(normalized),
    };

    // Invertir si está activado
    if inverted {
        normalized = 1.0 - normalized;
    }

    // Detectar fondo real (valores muy negativos o exactamente el mínimo)
    if val <= min_val + 0.001 {
        return background_color.to_rgb();
    }

    let intensity = (normalized * 255.0) as u32;
    (intensity << 16) | (intensity << 8) | intensity
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Abre archivo FITS
    let mut fptr = FitsFile::open("h_m51_b_s05_drz_sci.fits")?;
    let hdu = fptr.primary_hdu()?;

    // Obtiene dimensiones de la imagen
    let (img_width, img_height) = match &hdu.info {
        fitsio::hdu::HduInfo::ImageInfo { shape, .. } => (shape[1], shape[0]),
        _ => panic!("No es una imagen FITS"),
    };

    println!("Dimensiones de la imagen: {}x{}", img_width, img_height);
    println!("Cargando imagen completa en memoria...");

    // Carga toda la imagen en memoria de una sola vez
    let image_data: Vec<f32> = hdu.read_image(&mut fptr)?;

    // Calcula estadísticas de la imagen
    let mut min_val = f32::MAX;
    let mut max_val = f32::MIN;
    let mut sum = 0.0;
    let mut count_positive = 0;
    let mut count_zero_or_negative = 0;

    for &v in &image_data {
        min_val = min_val.min(v);
        max_val = max_val.max(v);
        sum += v;
        if v > 0.0 {
            count_positive += 1;
        } else {
            count_zero_or_negative += 1;
        }
    }

    let mean_val = sum / image_data.len() as f32;
    let range = max_val - min_val;

    println!("Imagen cargada:");
    println!(
        "  Min: {:.6}, Max: {:.6}, Rango: {:.6}",
        min_val, max_val, range
    );
    println!("  Media: {:.6}", mean_val);
    println!(
        "  Píxeles positivos: {}, Píxeles ≤0: {}",
        count_positive, count_zero_or_negative
    );

    // Configuración inicial de la ventana
    let mut win_width = 800;
    let mut win_height = 600;
    let mut window = Window::new(
        "FITS Viewer - /:Info | M:Modo | H:Auto | I:Invertir | Q/A:Brillo | E/D:Contraste | T/G:Gamma | R:Rotar | Space:Fondo",
        win_width,
        win_height,
        WindowOptions {
            resize: true,
            ..WindowOptions::default()
        },
    )?;

    // Estado de zoom y desplazamiento
    let mut zoom: f32 = 1.0;
    let mut offset_x: f32 = 0.0;
    let mut offset_y: f32 = 0.0;

    // Estado de rotación (0 = normal, 1 = 90°, 2 = 180°, 3 = 270°)
    let mut rotation: u8 = 0;
    let mut r_pressed = false;

    // Buffer de la ventana
    let mut buffer: Vec<u32> = vec![0; win_width * win_height];

    // Variables para controlar el framerate y movimiento suave
    let zoom_factor = 1.05;
    let pan_speed = 20.0;

    // Estado del color de fondo
    let mut background_color = BackgroundColor::Black;
    let mut space_pressed = false;

    // Controles de procesamiento de imagen
    let mut processing_mode = ProcessingMode::Linear;
    let mut brightness: f32 = 0.0; // -0.5 a 0.5
    let mut contrast: f32 = 1.0; // 0.1 a 3.0
    let mut gamma: f32 = 1.0; // 0.1 a 3.0
    let mut inverted = false;

    // Estados de teclas para evitar repetición
    let mut m_pressed = false;
    let mut i_pressed = false;
    let mut h_pressed = false;
    let mut question_pressed = false;

    // Estado del overlay de información
    let mut show_info = false;

    // Variables para seguimiento del mouse
    let mut mouse_x: usize = 0;
    let mut mouse_y: usize = 0;
    let mut mouse_img_x: usize = 0;
    let mut mouse_img_y: usize = 0;
    let mut mouse_pixel_val: f32 = 0.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Verificar si el tamaño de la ventana cambió
        let (new_width, new_height) = window.get_size();
        if new_width != win_width || new_height != win_height {
            win_width = new_width;
            win_height = new_height;
            buffer.resize(win_width * win_height, 0);
            println!("Ventana redimensionada a: {}x{}", win_width, win_height);
        }

        // Obtener posición del mouse
        if let Some((mx, my)) = window.get_mouse_pos(minifb::MouseMode::Clamp) {
            mouse_x = mx as usize;
            mouse_y = my as usize;

            // Convertir coordenadas de pantalla a coordenadas de imagen
            let img_x_f = (mouse_x as f32 / zoom + offset_x).max(0.0);
            let img_y_f = (mouse_y as f32 / zoom + offset_y).max(0.0);

            // Obtener dimensiones considerando rotación
            let (current_img_width, current_img_height) =
                get_rotated_dimensions(img_width, img_height, rotation);

            // Coordenadas en la imagen rotada
            let rotated_x = img_x_f as usize;
            let rotated_y = img_y_f as usize;

            if rotated_x < current_img_width && rotated_y < current_img_height {
                // Convertir a coordenadas originales
                let (orig_x, orig_y) = get_rotated_coords(
                    rotated_x,
                    rotated_y,
                    current_img_width,
                    current_img_height,
                    rotation,
                );

                if orig_x < img_width && orig_y < img_height {
                    mouse_img_x = orig_x;
                    mouse_img_y = orig_y;
                    let pixel_index = orig_y * img_width + orig_x;
                    mouse_pixel_val = image_data[pixel_index];
                }
            }
        }

        // Obtener dimensiones actuales considerando la rotación
        let (current_img_width, current_img_height) =
            get_rotated_dimensions(img_width, img_height, rotation);

        // Manejo de teclas para zoom y pan con movimiento más suave
        if window.is_key_down(Key::Up) {
            offset_y = (offset_y - pan_speed / zoom).max(0.0);
        }
        if window.is_key_down(Key::Down) {
            offset_y = (offset_y + pan_speed / zoom)
                .min((current_img_height as f32 - win_height as f32 / zoom).max(0.0));
        }
        if window.is_key_down(Key::Left) {
            offset_x = (offset_x - pan_speed / zoom).max(0.0);
        }
        if window.is_key_down(Key::Right) {
            offset_x = (offset_x + pan_speed / zoom)
                .min((current_img_width as f32 - win_width as f32 / zoom).max(0.0));
        }
        if window.is_key_down(Key::W) {
            zoom *= zoom_factor;
        }
        if window.is_key_down(Key::S) {
            zoom = (zoom / zoom_factor).max(0.1);
        }

        // Manejo de la tecla R para rotar
        if window.is_key_down(Key::R) {
            if !r_pressed {
                rotation = (rotation + 1) % 4;
                let angle = rotation as u16 * 90;
                println!("Imagen rotada {}°", angle);
                // Ajustar offsets después de la rotación para mantener la vista centrada
                offset_x = 0.0;
                offset_y = 0.0;
                r_pressed = true;
            }
        } else {
            r_pressed = false;
        }

        // Manejo de la tecla espacio para cambiar el fondo
        if window.is_key_down(Key::Space) {
            if !space_pressed {
                background_color = background_color.next();
                println!("Fondo cambiado a: {}", background_color.name());
                space_pressed = true;
            }
        } else {
            space_pressed = false;
        }

        // Manejo de teclas para procesamiento de imagen
        if window.is_key_down(Key::M) {
            if !m_pressed {
                processing_mode = processing_mode.next();
                println!("Modo de procesamiento: {}", processing_mode.name());
                m_pressed = true;
            }
        } else {
            m_pressed = false;
        }

        // Tecla I para invertir colores
        if window.is_key_down(Key::I) {
            if !i_pressed {
                inverted = !inverted;
                println!("Colores invertidos: {}", if inverted { "Sí" } else { "No" });
                i_pressed = true;
            }
        } else {
            i_pressed = false;
        }

        // Tecla H para auto-stretch (realce automático)
        if window.is_key_down(Key::H) {
            if !h_pressed {
                // Resetear valores para auto-stretch
                brightness = 0.0;
                contrast = 2.0; // Aumentar contraste automáticamente
                gamma = 0.7; // Gamma más bajo para realzar detalles
                println!(
                    "Auto-realce aplicado (Contraste: {:.1}, Gamma: {:.1})",
                    contrast, gamma
                );
                h_pressed = true;
            }
        } else {
            h_pressed = false;
        }

        // Controles de brillo con Q/A
        if window.is_key_down(Key::Q) {
            brightness = (brightness + 0.01).min(0.5);
        }
        if window.is_key_down(Key::A) {
            brightness = (brightness - 0.01).max(-0.5);
        }

        // Controles de contraste con E/D
        if window.is_key_down(Key::E) {
            contrast = (contrast + 0.05).min(3.0);
        }
        if window.is_key_down(Key::D) {
            contrast = (contrast - 0.05).max(0.1);
        }

        // Controles de gamma con T/G
        if window.is_key_down(Key::T) {
            gamma = (gamma + 0.05).min(3.0);
        }
        if window.is_key_down(Key::G) {
            gamma = (gamma - 0.05).max(0.1);
        }

        // Tecla / para mostrar/ocultar información
        if window.is_key_down(Key::Enter) {
            if !question_pressed {
                show_info = !show_info;
                println!(
                    "Overlay de información: {}",
                    if show_info { "Activado" } else { "Desactivado" }
                );
                question_pressed = true;
            }
        } else {
            question_pressed = false;
        }

        // Renderizado optimizado con soporte para rotación
        for y in 0..win_height {
            for x in 0..win_width {
                // Calcular coordenadas en la imagen rotada
                let rotated_x = ((x as f32 / zoom + offset_x) as usize)
                    .min(current_img_width.saturating_sub(1));
                let rotated_y = ((y as f32 / zoom + offset_y) as usize)
                    .min(current_img_height.saturating_sub(1));

                // Convertir a coordenadas originales de la imagen
                let (orig_x, orig_y) = get_rotated_coords(
                    rotated_x,
                    rotated_y,
                    current_img_width,
                    current_img_height,
                    rotation,
                );

                // Verificar que las coordenadas estén dentro de los límites
                if orig_x < img_width && orig_y < img_height {
                    let pixel_index = orig_y * img_width + orig_x;
                    let pixel_val = image_data[pixel_index];
                    buffer[y * win_width + x] = grayscale_to_rgb(
                        pixel_val,
                        min_val,
                        max_val,
                        background_color,
                        processing_mode,
                        brightness,
                        contrast,
                        gamma,
                        inverted,
                    );
                } else {
                    // Pixel fuera de los límites - usar color de fondo
                    buffer[y * win_width + x] = background_color.to_rgb();
                }
            }
        }

        // Renderizar overlay de información si está activado
        if show_info {
            // Verificar que tenemos espacio para el overlay
            if win_width > 320 && win_height > 270 {
                // Fondo oscuro sólido para el texto (gris oscuro) - más grande
                draw_rect(&mut buffer, 10, 10, 300, 250, win_width, 0x202020);

                // Marco para mejor visibilidad
                draw_rect(&mut buffer, 8, 8, 304, 1, win_width, 0xFFFFFF); // Top
                draw_rect(&mut buffer, 8, 259, 304, 1, win_width, 0xFFFFFF); // Bottom
                draw_rect(&mut buffer, 8, 8, 1, 252, win_width, 0xFFFFFF); // Left
                draw_rect(&mut buffer, 311, 8, 1, 252, win_width, 0xFFFFFF); // Right

                // Información simplificada usando solo números
                let info_values = vec![
                    format!("{:.2}", zoom),
                    format!("{:.0} {:.0}", offset_x, offset_y),
                    format!("{}", processing_mode.name()),
                    format!("{}", background_color.name()),
                    format!("{:.2}", brightness),
                    format!("{:.2}", contrast),
                    format!("{:.2}", gamma),
                    format!("{}", if inverted { "1" } else { "0" }),
                    format!("{}°", rotation as u16 * 90),
                    format!("{}x{}", win_width, win_height),
                    format!("{}x{}", img_width, img_height),
                    format!("{:.1} {:.1}", min_val, max_val),
                    format!("{} {}", mouse_x, mouse_y),
                    format!("{} {}", mouse_img_x, mouse_img_y),
                    format!("{:.2}", mouse_pixel_val),
                ];

                // Dibujar cada línea de información con solo números y símbolos
                for (i, value) in info_values.iter().enumerate() {
                    if i < 15 {
                        // Limitar a 12 líneas
                        // Filtrar solo caracteres que podemos dibujar
                        let simple_value = value
                            .chars()
                            .filter(|&c| matches!(c, '0'..='9' | '.' | '-' | ':' | ' ' | 'x' | '°'))
                            .collect::<String>();

                        draw_text(
                            &mut buffer,
                            &simple_value,
                            15,
                            15 + i * 15,
                            win_width,
                            0x00FF00, // Verde brillante
                        );
                    }
                }
            }
        }

        window.update_with_buffer(&buffer, win_width, win_height)?;
    }

    Ok(())
}
