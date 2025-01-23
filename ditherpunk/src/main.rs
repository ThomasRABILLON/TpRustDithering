use argh::FromArgs;
use image::io::Reader as ImageReader;
use image::{DynamicImage, ImageError};

#[derive(Debug, Clone, PartialEq, FromArgs)]
/// Convertit une image en monochrome ou vers une palette réduite de couleurs.
struct DitherArgs {
    /// le fichier d’entrée
    #[argh(positional)]
    input: String,

    /// le fichier de sortie (optionnel)
    #[argh(positional)]
    output: Option<String>,
    /// le mode d’opération
    #[argh(subcommand)]
    mode: Mode
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand)]
enum Mode {
    Seuil(OptsSeuil),
    Palette(OptsPalette),
    Couleurs(OptsCouleurs)
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="seuil")]
/// Rendu de l’image par seuillage monochrome.
struct OptsSeuil {}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="palette")]
/// Rendu de l’image avec une palette contenant un nombre limité de couleurs
struct OptsPalette {

    /// le nombre de couleurs à utiliser, dans la liste [NOIR, BLANC, ROUGE, VERT, BLEU, JAUNE, CYAN, MAGENTA]
    #[argh(option)]
    n_couleurs: usize
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="couleurs")]
/// Options for rendering the image with a specific list of colors.
struct OptsCouleurs {
    /// la liste des couleurs à utiliser, séparées par des virgules
    #[argh(option)]
    couleurs: String
}

const WHITE: image::Rgb<u8> = image::Rgb([255, 255, 255]);
const GREY: image::Rgb<u8> = image::Rgb([127, 127, 127]);
const BLACK: image::Rgb<u8> = image::Rgb([0, 0, 0]);
const BLUE: image::Rgb<u8> = image::Rgb([0, 0, 255]);
const RED: image::Rgb<u8> = image::Rgb([255, 0, 0]);
const GREEN: image::Rgb<u8> = image::Rgb([0, 255, 0]);
const YELLOW: image::Rgb<u8> = image::Rgb([255, 255, 0]);
const MAGENTA: image::Rgb<u8> = image::Rgb([255, 0, 255]);
const CYAN: image::Rgb<u8> = image::Rgb([0, 255, 255]);

fn white_pixel_1_out_of_2(rgb_image: &mut image::RgbImage) {
    for (x, _, pixel) in rgb_image.enumerate_pixels_mut() {
        if x % 2 == 0 {
            *pixel = WHITE;
        }
    }
    rgb_image
        .save_with_format("./output/1_pixel_blanc_sur_2.png", image::ImageFormat::Png)
        .unwrap();
}

fn get_luminance(pixel: &image::Rgb<u8>) -> f32 {
    let r = pixel[0] as f32;
    let g = pixel[1] as f32;
    let b = pixel[2] as f32;

    0.2126f32 * r + 0.7152f32 * g + 0.0722f32 * b
}

fn apply_threshold_seuillage(rgb_image: &mut image::RgbImage, couleur1: image::Rgb<u8>, couleur2: image::Rgb<u8>) {
    for (_x, _y, pixel) in rgb_image.enumerate_pixels_mut() {
        let luminance = get_luminance(pixel);

        if luminance > 127.5 {
            *pixel = couleur1;
        } else {
            *pixel = couleur2;
        }
    }
    rgb_image
        .save_with_format("./output/output_monochrome.png", image::ImageFormat::Png)
        .unwrap();
}

fn main() -> Result<(), ImageError> {
    let args: DitherArgs = argh::from_env();
    let path_in = args.input;
    let img: DynamicImage = ImageReader::open(path_in)?.decode()?;
    let rgb_image = img.to_rgb8();

    // Coordonnées du pixel (32, 52)
    let x = 32;
    let y = 52;

    // Vérifie si les coordonnées sont valides
    if x < rgb_image.width() && y < rgb_image.height() {
        // Récupère la couleur du pixel
        let pixel = rgb_image.get_pixel(x, y);
        println!("La couleur du pixel ({}, {}) est : {:?}", x, y, pixel);
    } else {
        println!(
            "Coordonnées ({}, {}) hors de l'image (dimensions : {}x{}).",
            x,
            y,
            rgb_image.width(),
            rgb_image.height()
        );
    }

    //on boucle sur chaque pixel de l'image
    white_pixel_1_out_of_2(&mut rgb_image.clone());

    //on sauvegarde l'image modifiée
    rgb_image.save_with_format("./output/output.png", image::ImageFormat::Png)?;

    // Appliquer le traitement de seuillage
    if let Mode::Couleurs(opts) = args.mode {
        let couleurs = opts.couleurs.split(",").collect::<Vec<&str>>();
        let mut couleurs_rgb = Vec::new();
        for couleur in couleurs {
            match couleur {
                "BLACK" => couleurs_rgb.push(BLACK),
                "WHITE" => couleurs_rgb.push(WHITE),
                "RED" => couleurs_rgb.push(RED),
                "GREEN" => couleurs_rgb.push(GREEN),
                "BLUE" => couleurs_rgb.push(BLUE),
                "YELLOW" => couleurs_rgb.push(YELLOW),
                "CYAN" => couleurs_rgb.push(CYAN),
                "MAGENTA" => couleurs_rgb.push(MAGENTA),
                _ => couleurs_rgb.push(GREY),
            }
        }
        apply_threshold_seuillage(&mut rgb_image.clone(), couleurs_rgb[0], couleurs_rgb[1]);
    } else {
        apply_threshold_seuillage(&mut rgb_image.clone(), BLACK, WHITE);
    }

    // Sauvegarder l'image traitée
    // rgb_image.save_with_format("./output/output_monochrome.png", image::ImageFormat::Png)?;

    Ok(())
}
