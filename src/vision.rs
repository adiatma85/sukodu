use ab_glyph::{Font, FontRef, PxScale};
use image::{GrayImage, ImageFormat, Luma, Rgb, RgbImage, imageops};
use imageproc::contours::find_contours;
use imageproc::contrast::adaptive_threshold;
use imageproc::geometric_transformations::{Interpolation, Projection, warp};
use imageproc::point::Point;
use leptess::{LepTess, Variable};
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::process::Command;

/// Ensures the Arimo font is downloaded to resources/font.ttf.
pub fn ensure_font() -> Result<(), Box<dyn std::error::Error>> {
    let resources_dir = Path::new("resources");
    if !resources_dir.exists() {
        fs::create_dir_all(resources_dir)?;
    }

    let font_path = resources_dir.join("font.ttf");
    if !font_path.exists() {
        println!("Downloading Arimo font for OCR templates...");
        let status = Command::new("curl")
            .args([
                "-L",
                "-o",
                font_path.to_str().unwrap(),
                "https://github.com/google/fonts/raw/main/ofl/arimo/Arimo%5Bwght%5D.ttf",
            ])
            .status()?;
        if !status.success() {
            return Err("Failed to download font using curl".into());
        }
    }
    Ok(())
}

/// Load image from file and resize to exactly 800x800 for consistent processing.
pub fn load_image<P: AsRef<Path>>(path: P) -> Result<GrayImage, Box<dyn std::error::Error>> {
    let img = image::open(path)?.to_luma8();
    let resized = imageops::resize(&img, 800, 800, imageops::FilterType::Lanczos3);
    Ok(resized)
}

/// Find the Sudoku grid's corners. Returns [top_left, top_right, bottom_right, bottom_left].
pub fn find_grid_corners(
    binary: &GrayImage,
) -> Result<[Point<i32>; 4], Box<dyn std::error::Error>> {
    let mut inverted = binary.clone();
    for p in inverted.iter_mut() {
        *p = 255 - *p;
    }
    let contours = find_contours::<i32>(&inverted);

    let mut best_contour = None;
    let mut max_area = 0;

    for contour in contours {
        let points = &contour.points;
        if points.len() < 4 {
            continue;
        }

        // Find bounding box
        let mut min_x = i32::MAX;
        let mut max_x = i32::MIN;
        let mut min_y = i32::MAX;
        let mut max_y = i32::MIN;

        for p in points {
            if p.x < min_x {
                min_x = p.x;
            }
            if p.x > max_x {
                max_x = p.x;
            }
            if p.y < min_y {
                min_y = p.y;
            }
            if p.y > max_y {
                max_y = p.y;
            }
        }

        let width = max_x - min_x;
        let height = max_y - min_y;

        if width < 150 || height < 150 {
            continue; // too small
        }

        // Reject contours that touch the image boundary (800x800)
        if min_x < 5 || min_y < 5 || max_x > 795 || max_y > 795 {
            continue;
        }

        let aspect_ratio = width as f32 / height as f32;
        if !(0.75..=1.35).contains(&aspect_ratio) {
            continue; // not square enough
        }

        let area = width * height;
        if area > max_area {
            max_area = area;
            best_contour = Some(points.clone());
        }
    }

    let points = best_contour.ok_or("Could not locate Sudoku grid contour in the image")?;

    // Find corners
    // Top-Left minimizes x + y
    // Bottom-Right maximizes x + y
    // Top-Right maximizes x - y
    // Bottom-Left minimizes x - y
    let mut tl = points[0];
    let mut tr = points[0];
    let mut br = points[0];
    let mut bl = points[0];

    let mut min_sum = i32::MAX;
    let mut max_sum = i32::MIN;
    let mut max_diff = i32::MIN;
    let mut min_diff = i32::MAX;

    for p in points {
        let sum = p.x + p.y;
        let diff = p.x - p.y;

        if sum < min_sum {
            min_sum = sum;
            tl = p;
        }
        if sum > max_sum {
            max_sum = sum;
            br = p;
        }
        if diff > max_diff {
            max_diff = diff;
            tr = p;
        }
        if diff < min_diff {
            min_diff = diff;
            bl = p;
        }
    }

    Ok([tl, tr, br, bl])
}

/// Warps the perspective of the grid to a clean 576x576 square.
pub fn warp_grid(
    gray: &GrayImage,
    corners: [Point<i32>; 4],
) -> Result<GrayImage, Box<dyn std::error::Error>> {
    let src = [
        (corners[0].x as f32 + 4.0, corners[0].y as f32 + 4.0),
        (corners[1].x as f32 - 4.0, corners[1].y as f32 + 4.0),
        (corners[2].x as f32 - 4.0, corners[2].y as f32 - 4.0),
        (corners[3].x as f32 + 4.0, corners[3].y as f32 - 4.0),
    ];

    let dst = [(0.0, 0.0), (576.0, 0.0), (576.0, 576.0), (0.0, 576.0)];

    let proj = Projection::from_control_points(src, dst)
        .ok_or("Collinear corners, cannot calculate projection")?;
    let warped = warp(
        gray,
        proj,
        Interpolation::Bilinear,
        imageproc::geometric_transformations::Border::Constant(Luma([0u8])),
    );

    // Crop the top-left 576x576 region
    let cropped = imageops::crop_imm(&warped, 0, 0, 576, 576).to_image();
    Ok(cropped)
}

/// Detects if the warped grid is 9x9 or 16x16 using grid line density.
pub fn detect_grid_size(warped_gray: &GrayImage) -> usize {
    // Apply local thresholding to highlight lines
    let binary = adaptive_threshold(warped_gray, 5, 10);

    // Sum white pixels along X and Y axes
    let mut proj_x = vec![0.0; 576];
    let mut proj_y = vec![0.0; 576];

    for y in 0..576 {
        for x in 0..576 {
            // Invert so lines/digits are white (value 255)
            let val = 255 - binary.get_pixel(x, y)[0];
            if val > 128 {
                proj_x[x as usize] += 1.0;
                proj_y[y as usize] += 1.0;
            }
        }
    }

    let autocorr = |proj: &[f64], lag: usize| -> f64 {
        let n = proj.len();
        if lag >= n {
            return 0.0;
        }
        let mean = proj.iter().sum::<f64>() / n as f64;
        let mut sum = 0.0;
        let count = n - lag;
        for i in 0..count {
            sum += (proj[i] - mean) * (proj[i + lag] - mean);
        }
        sum / count as f64
    };

    let acf_x_9 = autocorr(&proj_x, 64);
    let acf_y_9 = autocorr(&proj_y, 64);
    let score_9 = acf_x_9 + acf_y_9;

    let acf_x_16 = autocorr(&proj_x, 36);
    let acf_y_16 = autocorr(&proj_y, 36);
    let score_16 = acf_x_16 + acf_y_16;

    if score_9 > score_16 { 9 } else { 16 }
}

fn clear_border_components(binary: &mut GrayImage) {
    let w = binary.width();
    let h = binary.height();
    let mut visited = vec![vec![false; h as usize]; w as usize];

    for y in 0..h {
        for x in 0..w {
            if binary.get_pixel(x, y)[0] > 128 && !visited[x as usize][y as usize] {
                let mut component = Vec::new();
                let mut touches_border = false;
                let mut queue = vec![(x, y)];
                visited[x as usize][y as usize] = true;

                let mut head = 0;
                while head < queue.len() {
                    let (cx, cy) = queue[head];
                    head += 1;
                    component.push((cx, cy));

                    if cx == 0 || cx == w - 1 || cy == 0 || cy == h - 1 {
                        touches_border = true;
                    }

                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let nx = cx as i32 + dx;
                            let ny = cy as i32 + dy;
                            if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                                let ux = nx as u32;
                                let uy = ny as u32;
                                if binary.get_pixel(ux, uy)[0] > 128
                                    && !visited[ux as usize][uy as usize]
                                {
                                    visited[ux as usize][uy as usize] = true;
                                    queue.push((ux, uy));
                                }
                            }
                        }
                    }
                }

                if touches_border {
                    for (cx, cy) in component {
                        binary.put_pixel(cx, cy, Luma([0u8]));
                    }
                }
            }
        }
    }
}

/// Builds the set of characters that may appear in a grid of the given size:
/// `1..=9` plus `A..` for values `10..=size` (e.g. 16x16 uses `A..G`).
fn charset_for_size(size: usize) -> String {
    let mut s = String::new();
    for v in 1..=size {
        if v <= 9 {
            s.push((b'0' + v as u8) as char);
        } else {
            s.push((b'A' + (v - 10) as u8) as char);
        }
    }
    s
}

/// Maps the first recognized character in `text` back to a cell value, or `None` if the
/// text holds no character valid for this grid size.
fn ocr_char_to_value(text: &str, size: usize) -> Option<usize> {
    for ch in text.chars() {
        let c = ch.to_ascii_uppercase();
        if c.is_ascii_digit() && c != '0' {
            let v = (c as u8 - b'0') as usize;
            if v <= size {
                return Some(v);
            }
        } else if c.is_ascii_uppercase() {
            let v = (c as u8 - b'A') as usize + 10;
            if v <= size {
                return Some(v);
            }
        }
    }
    None
}

/// Turns a binary cell (white digit on black) into the black-on-white image Tesseract reads
/// most reliably: the digit keeps its position, gets a generous white border so it is clear
/// of the edges, and is upscaled to a fixed, comfortably large frame. (A parameter sweep
/// against synthetic puzzles found this plain layout beats bounding-box normalisation and
/// post-blur for single-character recognition.)
fn prepare_cell_for_ocr(cell_bin: &GrayImage) -> GrayImage {
    const OUT: u32 = 200;
    let inner = cell_bin.width();
    let pad = inner / 2;
    let canvas_size = inner + pad * 2;

    // White background; paint the digit black.
    let mut canvas = GrayImage::from_pixel(canvas_size, canvas_size, Luma([255u8]));
    for (x, y, p) in cell_bin.enumerate_pixels() {
        if p[0] > 128 {
            canvas.put_pixel(x + pad, y + pad, Luma([0u8]));
        }
    }

    imageops::resize(&canvas, OUT, OUT, imageops::FilterType::Lanczos3)
}

/// Extracts each cell from the warped grid and recognizes its digit with Tesseract OCR.
///
/// The grid is divided into `size * size` cells; the centre of each cell is cropped (to
/// avoid the grid lines), binarized, cleaned of border-touching components, and — if it
/// holds enough ink to be non-empty — passed to Tesseract in single-character mode with a
/// whitelist restricted to the valid Sudoku alphabet. Empty cells are left as `0`.
///
/// Returns an error if Tesseract cannot be initialised (e.g. it is not installed).
pub fn scan_board(
    warped_gray: &GrayImage,
    size: usize,
) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    // One engine, reused for every cell.
    let mut tess = LepTess::new(None, "eng").map_err(|e| {
        format!(
            "Tesseract could not be initialised ({e}). Install it first — e.g. \
             `brew install tesseract leptonica` on macOS, or \
             `apt-get install tesseract-ocr libtesseract-dev libleptonica-dev` on Debian/Ubuntu."
        )
    })?;

    // PSM 10 = treat the image as a single character; whitelist the valid alphabet.
    tess.set_variable(Variable::TesseditPagesegMode, "10")?;
    tess.set_variable(Variable::TesseditCharWhitelist, &charset_for_size(size))?;

    let cell_size = 576 / size;
    // Crop most of each cell. Including the grid lines is fine — `clear_border_components`
    // removes anything touching the crop edge — and the wider crop keeps the digit clear of
    // the border so it is never clipped.
    let crop_size = if size == 9 { 56 } else { 32 };
    let margin = (cell_size - crop_size) / 2;

    // Upscale every cell to a common working resolution before binarizing. Interpolating the
    // grayscale first gives smooth glyph edges that the OCR reads far more reliably than a
    // coarse native-resolution threshold (confirmed by a parameter sweep).
    const WORK_SIZE: u32 = 96;
    let area = (WORK_SIZE * WORK_SIZE) as f32;

    let mut board = vec![0; size * size];

    for row in 0..size {
        for col in 0..size {
            let cx = (col * cell_size + margin) as u32;
            let cy = (row * cell_size + margin) as u32;
            let cell = imageops::crop_imm(warped_gray, cx, cy, crop_size as u32, crop_size as u32)
                .to_image();
            let work =
                imageops::resize(&cell, WORK_SIZE, WORK_SIZE, imageops::FilterType::Lanczos3);

            // Local contrast: a low spread means the cell is empty.
            let (mut min_val, mut max_val) = (255u8, 0u8);
            for p in work.iter() {
                if *p < min_val {
                    min_val = *p;
                }
                if *p > max_val {
                    max_val = *p;
                }
            }
            if max_val - min_val < 35 {
                continue;
            }

            // Binarize so the digit is white (255) on a black (0) background.
            let thresh = min_val + (max_val - min_val) / 2;
            let mut cell_bin = GrayImage::new(WORK_SIZE, WORK_SIZE);
            for (x, y, p) in work.enumerate_pixels() {
                cell_bin.put_pixel(x, y, Luma([if p[0] < thresh { 255 } else { 0 }]));
            }

            // Drop components touching the border (residual grid lines).
            clear_border_components(&mut cell_bin);

            // Reject cells with too little / too much ink to be a single digit.
            let white_pixels = cell_bin.pixels().filter(|p| p[0] > 128).count() as f32;
            if white_pixels < area * 0.015 || white_pixels > area * 0.5 {
                continue;
            }

            // Hand the cleaned-up cell to Tesseract as an in-memory PNG.
            let ocr_img = prepare_cell_for_ocr(&cell_bin);
            let mut buf = Vec::new();
            ocr_img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)?;

            tess.set_image_from_mem(&buf)?;
            tess.set_source_resolution(70); // silence the "invalid resolution" warning
            tess.recognize();
            let text = tess.get_utf8_text().unwrap_or_default();

            if std::env::var("SUKODU_OCR_DEBUG").is_ok() {
                eprintln!(
                    "cell ({},{}) ink={} -> {:?}",
                    row,
                    col,
                    white_pixels as usize,
                    text.trim()
                );
            }

            if let Some(val) = ocr_char_to_value(&text, size) {
                board[row * size + col] = val;
            }
        }
    }

    Ok(board)
}

/// Renders the solved Sudoku digits onto the warped grid image.
///
/// Takes the warped grayscale image, the original scanned board (to identify empty cells),
/// the solved board, and the grid size. Returns an RGB image with the solved cells filled
/// in with a distinct color.
pub fn draw_solution(
    warped_gray: &GrayImage,
    original_board: &[usize],
    solved_board: &[usize],
    size: usize,
) -> Result<RgbImage, Box<dyn std::error::Error>> {
    ensure_font()?;

    let font_data = fs::read("resources/font.ttf")?;
    let font = FontRef::try_from_slice(&font_data)?;

    let mut rgb_img = RgbImage::new(warped_gray.width(), warped_gray.height());
    for (x, y, p) in warped_gray.enumerate_pixels() {
        let val = p[0];
        rgb_img.put_pixel(x, y, Rgb([val, val, val]));
    }

    let cell_size = 576 / size;
    let scale = PxScale::from((cell_size as f32 * 0.65).round());

    for row in 0..size {
        for col in 0..size {
            let idx = row * size + col;
            // Only draw digits that were empty (0) in the original board
            if original_board[idx] == 0 && solved_board[idx] != 0 {
                let val = solved_board[idx];
                let c = if (1..=9).contains(&val) {
                    (b'0' + val as u8) as char
                } else if (10..=16).contains(&val) {
                    (b'A' + (val - 10) as u8) as char
                } else {
                    continue;
                };

                let glyph = font.glyph_id(c).with_scale(scale);
                if let Some(outline) = font.outline_glyph(glyph) {
                    let bounds = outline.px_bounds();
                    let gw = bounds.width();
                    let gh = bounds.height();

                    // Center cell position in the warped grid (576x576)
                    let cell_cx = col * cell_size + cell_size / 2;
                    let cell_cy = row * cell_size + cell_size / 2;

                    let pad_x = cell_cx as f32 - gw / 2.0;
                    let pad_y = cell_cy as f32 - gh / 2.0;

                    // Choose a nice vibrant accent color (e.g. blue: [0, 102, 204])
                    let accent_color = [0, 102, 204];

                    outline.draw(|x, y, v| {
                        let px = (x as f32 + pad_x).round() as i32;
                        let py = (y as f32 + pad_y).round() as i32;
                        if (0..576).contains(&px) && (0..576).contains(&py) {
                            let bg_pixel = rgb_img.get_pixel(px as u32, py as u32);
                            let bg_r = bg_pixel[0] as f32;
                            let bg_g = bg_pixel[1] as f32;
                            let bg_b = bg_pixel[2] as f32;

                            let r = (v * accent_color[0] as f32 + (1.0 - v) * bg_r).round() as u8;
                            let g = (v * accent_color[1] as f32 + (1.0 - v) * bg_g).round() as u8;
                            let b = (v * accent_color[2] as f32 + (1.0 - v) * bg_b).round() as u8;

                            rgb_img.put_pixel(px as u32, py as u32, Rgb([r, g, b]));
                        }
                    });
                }
            }
        }
    }

    Ok(rgb_img)
}

/// Programmatically generate a synthetic Sudoku grid image for testing.
pub fn generate_synthetic_image<P: AsRef<Path>>(
    path: P,
    size: usize,
    board: &[usize],
) -> Result<(), Box<dyn std::error::Error>> {
    ensure_font()?;

    let font_data = fs::read("resources/font.ttf")?;
    let font = FontRef::try_from_slice(&font_data)?;

    // Create an 800x800 white image
    let mut img = GrayImage::from_pixel(800, 800, Luma([255u8]));

    // Draw outer border and grid lines
    let border = 50;
    let grid_area = 700;
    let cell_size = grid_area / size;
    let actual_grid_area = size * cell_size;

    // Draw outer grid lines
    for i in 0..=size {
        let pos = border + i * cell_size;
        let is_block_border = i % (size as f64).sqrt().floor() as usize == 0;
        let thickness = if is_block_border { 4 } else { 1 };

        let max_t = 4;

        // Horizontal line
        for dy in -thickness..=thickness {
            let y = pos as i32 + dy;
            if (0..800).contains(&y) {
                for x in (border - max_t)..(border + actual_grid_area + max_t) {
                    img.put_pixel(x as u32, y as u32, Luma([0u8]));
                }
            }
        }

        // Vertical line
        for dx in -thickness..=thickness {
            let x = pos as i32 + dx;
            if (0..800).contains(&x) {
                for y in (border - max_t)..(border + actual_grid_area + max_t) {
                    img.put_pixel(x as u32, y as u32, Luma([0u8]));
                }
            }
        }
    }

    // Draw digits
    let scale = PxScale::from((cell_size as f32 * 0.6).round());
    for row in 0..size {
        for col in 0..size {
            let val = board[row * size + col];
            if val == 0 {
                continue;
            }

            let c = if (1..=9).contains(&val) {
                (b'0' + val as u8) as char
            } else if (10..=16).contains(&val) {
                (b'A' + (val - 10) as u8) as char
            } else {
                continue;
            };

            let glyph = font.glyph_id(c).with_scale(scale);
            if let Some(outline) = font.outline_glyph(glyph) {
                let bounds = outline.px_bounds();
                let gw = bounds.width();
                let gh = bounds.height();

                // Center cell position
                let cell_cx = border + col * cell_size + cell_size / 2;
                let cell_cy = border + row * cell_size + cell_size / 2;

                let pad_x = cell_cx as f32 - gw / 2.0;
                let pad_y = cell_cy as f32 - gh / 2.0;

                outline.draw(|x, y, v| {
                    let px = (x as f32 + pad_x).round() as i32;
                    let py = (y as f32 + pad_y).round() as i32;
                    if (0..800).contains(&px) && (0..800).contains(&py) {
                        let val = (255.0 * (1.0 - v)) as u8;
                        if val < 128 {
                            img.put_pixel(px as u32, py as u32, Luma([val]));
                        }
                    }
                });
            }
        }
    }

    img.save(path)?;
    Ok(())
}
