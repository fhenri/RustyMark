use std::path::{Path, PathBuf};
use std::fs;
use std::error::Error;

use image::{DynamicImage, GenericImageView, Rgba};
use imageproc::drawing::{draw_text_mut, text_size};
use rusttype::{Font, Scale};
use serde::Deserialize;

// Configuration structure for copyright settings
#[derive(Debug, Clone, Deserialize)]
struct CopyrightConfig {
    #[serde(default = "default_text")]
    text: String,
    
    #[serde(default = "default_font_path")]
    font_path: PathBuf,
    
    #[serde(default = "default_font_size")]
    font_size: f32,
    
    #[serde(default = "default_position")]
    position: Position,
    
    #[serde(default = "default_color")]
    color: ColorConfig,
}

// Default value functions
fn default_text() -> String {
    "© Copyright".to_string()
}

fn default_font_path() -> PathBuf {
    PathBuf::from("/path/to/default/font.ttf")
}

fn default_font_size() -> f32 {
    20.0
}

fn default_position() -> Position {
    Position::BottomRight
}

fn default_color() -> ColorConfig {
    ColorConfig {
        r: 255,
        g: 255,
        b: 255,
        a: 128,
    }
}

// Separate struct for color configuration
#[derive(Debug, Clone, Deserialize)]
struct ColorConfig {
    #[serde(default = "default_color_component")]
    r: u8,
    #[serde(default = "default_color_component")]
    g: u8,
    #[serde(default = "default_color_component")]
    b: u8,
    #[serde(default = "default_alpha")]
    a: u8,
}

fn default_color_component() -> u8 {
    255
}

fn default_alpha() -> u8 {
    128
}

// Enum for positioning the copyright text
#[derive(Debug, Clone, PartialEq, Deserialize)]
enum Position {
    #[serde(rename = "top_left")]
    TopLeft,
    #[serde(rename = "top_center")]
    TopCenter,
    #[serde(rename = "top_right")]
    TopRight,
    #[serde(rename = "middle_left")]
    MiddleLeft,
    #[serde(rename = "middle_center")]
    MiddleCenter,
    #[serde(rename = "middle_right")]
    MiddleRight,
    #[serde(rename = "bottom_left")]
    BottomLeft,
    #[serde(rename = "bottom_center")]
    BottomCenter,
    #[serde(rename = "bottom_right")]
    BottomRight,
}

// Parse configuration from a TOML file
fn parse_config(config_path: &Path) -> Result<CopyrightConfig, Box<dyn Error>> {
    let config_content = fs::read_to_string(config_path)?;
    let config: CopyrightConfig = toml::from_str(&config_content)?;

    Ok(config)
}

// Calculate text position based on selected position
fn calculate_text_position(
    image: &DynamicImage, 
    text_width: u32, 
    text_height: u32, 
    position: &Position
) -> (i32, i32) {
    let (img_width, img_height) = image.dimensions();
    
    match position {
        Position::TopLeft => (10, 10),
        Position::TopCenter => ((img_width - text_width) as i32 / 2, 10),
        Position::TopRight => ((img_width - text_width) as i32 - 10, 10),
        Position::MiddleLeft => (10, (img_height - text_height) as i32 / 2),
        Position::MiddleCenter => (
            (img_width - text_width) as i32 / 2, 
            (img_height - text_height) as i32 / 2
        ),
        Position::MiddleRight => (
            (img_width - text_width) as i32 - 10, 
            (img_height - text_height) as i32 / 2
        ),
        Position::BottomLeft => (10, (img_height - text_height) as i32 - 10),
        Position::BottomCenter => (
            (img_width - text_width) as i32 / 2, 
            (img_height - text_height) as i32 - 10
        ),
        Position::BottomRight => (
            (img_width - text_width) as i32 - 10, 
            (img_height - text_height) as i32 - 10
        ),
    }
}

// Add copyright text to an image
fn add_copyright_text_image(
    image_path: &Path, 
    config: &CopyrightConfig
) -> Result<(), Box<dyn Error>> {
    // Load the font
    let font_data = fs::read(&config.font_path)?;
    let font = Font::try_from_vec(font_data)
        .ok_or("Error loading font")?;

    // Load the image
    let image = image::open(image_path)?;

    // Create scale for the font
    let scale = Scale::uniform(config.font_size);

    // Calculate text size
    let (text_width, text_height) = text_size(scale, &font, &config.text);

    // Calculate text position
    let (x, y) = calculate_text_position(&image, text_width as u32, text_height as u32, &config.position);

    // Convert image to RGBA if needed
    let mut rgba_image = image.to_rgba8();

    // Draw text with Unicode support
    draw_text_mut(
        &mut rgba_image, 
        Rgba([config.color.r, config.color.g, config.color.b, config.color.a]), 
        x, 
        y, 
        scale, 
        &font, 
        &config.text
    );

    // Save the modified image
    let output_path = image_path.with_file_name(
        format!("watermarked_{}", image_path.file_name().unwrap().to_str().unwrap())
    );
    rgba_image.save(output_path)?;

    Ok(())
}

// Process images in a directory or a single file
fn process_images(
    input_path: &Path, 
    config_path: &Path
) -> Result<(), Box<dyn Error>> {
    // Parse configuration
    let config = parse_config(config_path)?;

    // Check if input is a directory or a file
    if input_path.is_dir() {
        // Process all image files in the directory
        for entry in fs::read_dir(input_path)? {
            let entry = entry?;
            let path = entry.path();
            
            // Check if it's an image file
            if path.is_file() && is_image_file(&path) {
                // Add visual watermark
                if let Err(e) = add_copyright_text_image(&path, &config) {
                    eprintln!("Error processing visual watermark {}: {}", path.display(), e);
                }
                
                // Add metadata copyright
                /*
                if let Err(e) = add_copyright_metadata(&path, &config) {
                    eprintln!("Error processing metadata for {}: {}", path.display(), e);
                }
                */
            }
        }
    } else if input_path.is_file() && is_image_file(input_path) {
        // Process single image file
        add_copyright_text_image(input_path, &config)?;
    } else {
        return Err("Invalid input path".into());
    }

    Ok(())
}

// Helper function to check if a file is an image
fn is_image_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext = ext.to_str().unwrap_or("").to_lowercase();
        ["jpg", "jpeg", "png", "gif", "bmp", "webp"].contains(&ext.as_str())
    } else {
        false
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <image_file_or_directory> <config_file>", args[0]);
        std::process::exit(1);
    }

    // Process images
    process_images(Path::new(&args[1]), Path::new(&args[2]))?;

    println!("Copyright watermark added successfully!");
    Ok(())
}